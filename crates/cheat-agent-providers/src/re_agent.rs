use cheat_agent_core::{CheatError, DataType, PointerChain, Result, ScanMode};
use cheat_agent_ffi::{MemoryScanner, ProcessBackend};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetIdentity {
    pub pid: u32,
    pub executable_path: Option<String>,
    pub architecture: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceEnvelope {
    pub request_id: String,
    pub target_identity: Option<TargetIdentity>,
    pub timestamp: String,
    pub operation: String,
    pub status: String,
    pub observations: Value,
    pub artifacts: Vec<String>,
    pub limitations: Vec<String>,
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

pub struct ReAgentProvider {
    backend: Arc<Mutex<Option<ProcessBackend>>>,
    scanner: Arc<Mutex<Option<MemoryScanner>>>,
    target_identity: Arc<Mutex<Option<TargetIdentity>>>,
}

impl Default for ReAgentProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ReAgentProvider {
    pub fn new() -> Self {
        Self {
            backend: Arc::new(Mutex::new(None)),
            scanner: Arc::new(Mutex::new(None)),
            target_identity: Arc::new(Mutex::new(None)),
        }
    }

    pub fn attach(&self, request_id: &str, pid: u32, executable_path: Option<String>) -> Result<EvidenceEnvelope> {
        let backend = ProcessBackend::new(pid)?;
        if !backend.is_alive() {
            return Err(CheatError::NotConnected);
        }
        let scanner = MemoryScanner::new(&backend)?;

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };

        let session_id = format!("sess_{}_{}", pid, current_timestamp().replace('.', "_"));
        let ident = TargetIdentity {
            pid,
            executable_path: executable_path.clone(),
            architecture: arch.to_string(),
            session_id,
        };

        let is_alive = backend.is_alive();
        let has_gtlib = ProcessBackend::has_gtlibcpp();

        *self.backend.lock().unwrap() = Some(backend);
        *self.scanner.lock().unwrap() = Some(scanner);
        *self.target_identity.lock().unwrap() = Some(ident.clone());

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: Some(ident),
            timestamp: current_timestamp(),
            operation: "target.attach".to_string(),
            status: "success".to_string(),
            observations: json!({
                "pid": pid,
                "alive": is_alive,
                "gtlibcpp_backend": has_gtlib,
                "executable_path": executable_path
            }),
            artifacts: vec![],
            limitations: vec![
                "Target must remain alive for subsequent session calls".to_string(),
                "Memory layout changes on process restart".to_string(),
            ],
        })
    }

    pub fn inspect(&self, request_id: &str) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let is_alive = backend.is_alive();
        let regions = backend.query_regions().unwrap_or_default();
        let modules = backend.query_modules().unwrap_or_default();

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "target.inspect".to_string(),
            status: "success".to_string(),
            observations: json!({
                "alive": is_alive,
                "region_count": regions.len(),
                "module_count": modules.len(),
                "modules": modules.iter().map(|m| &m.name).collect::<Vec<_>>()
            }),
            artifacts: vec![],
            limitations: vec![
                "Module base addresses subject to ASLR on reload".to_string(),
            ],
        })
    }

    pub fn query_regions(&self, request_id: &str) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let regions = backend.query_regions()?;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.regions".to_string(),
            status: "success".to_string(),
            observations: json!({
                "region_count": regions.len(),
                "writable_region_count": regions.iter().filter(|r| r.is_writable).count(),
                "total_size_bytes": regions.iter().map(|r| r.size).sum::<u64>()
            }),
            artifacts: vec![],
            limitations: vec![
                "Region boundaries bounded by OS protections".to_string(),
            ],
        })
    }

    pub fn read_memory(&self, request_id: &str, address: u64, size: usize) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let bytes = backend.read_memory(address, size)?;
        let hex_view = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.read".to_string(),
            status: "success".to_string(),
            observations: json!({
                "address": format!("0x{:X}", address),
                "bytes_read": bytes.len(),
                "hex": hex_view,
                "raw_bytes": bytes
            }),
            artifacts: vec![],
            limitations: vec![
                "Read reflects snapshot at timestamp; target memory may mutate concurrently".to_string(),
            ],
        })
    }

    pub fn scan_first(&self, request_id: &str, data_type: DataType, scan_mode: ScanMode, val1: Option<&[u8]>, val2: Option<&[u8]>) -> Result<EvidenceEnvelope> {
        let mut s_guard = self.scanner.lock().unwrap();
        let scanner = s_guard.as_mut().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let count = scanner.first_scan(data_type, scan_mode, val1, val2)?;
        let matches = scanner.get_matches(100)?;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.scan.first".to_string(),
            status: "success".to_string(),
            observations: json!({
                "data_type": format!("{:?}", data_type),
                "scan_mode": format!("{:?}", scan_mode),
                "match_count": count,
                "sampled_matches": matches.iter().map(|a| format!("0x{:X}", a)).collect::<Vec<_>>()
            }),
            artifacts: vec![],
            limitations: vec![
                "Initial scan bounds candidates across writable pages; refinement required".to_string(),
            ],
        })
    }

    pub fn scan_next(&self, request_id: &str, scan_mode: ScanMode, val1: Option<&[u8]>, val2: Option<&[u8]>) -> Result<EvidenceEnvelope> {
        let mut s_guard = self.scanner.lock().unwrap();
        let scanner = s_guard.as_mut().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let count = scanner.next_scan(scan_mode, val1, val2)?;
        let matches = scanner.get_matches(100)?;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.scan.next".to_string(),
            status: "success".to_string(),
            observations: json!({
                "scan_mode": format!("{:?}", scan_mode),
                "remaining_matches": count,
                "sampled_matches": matches.iter().map(|a| format!("0x{:X}", a)).collect::<Vec<_>>()
            }),
            artifacts: vec![],
            limitations: vec![
                "Candidates reflect differential criteria since previous scan snapshot".to_string(),
            ],
        })
    }

    pub fn scan_aob(&self, request_id: &str, pattern: &str) -> Result<EvidenceEnvelope> {
        let mut s_guard = self.scanner.lock().unwrap();
        let scanner = s_guard.as_mut().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let count = scanner.scan_aob(pattern)?;
        let matches = scanner.get_matches(100)?;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.aob".to_string(),
            status: "success".to_string(),
            observations: json!({
                "pattern": pattern,
                "match_count": count,
                "matches": matches.iter().map(|a| format!("0x{:X}", a)).collect::<Vec<_>>()
            }),
            artifacts: vec![],
            limitations: vec![
                "Wildcard matches bounded to committed readable executable/data regions".to_string(),
            ],
        })
    }

    pub fn resolve_pointer(&self, request_id: &str, chain: &PointerChain) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let resolved_addr = backend.resolve_pointer(chain)?;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.pointer.resolve".to_string(),
            status: "success".to_string(),
            observations: json!({
                "base_module": chain.base_module,
                "base_offset": format!("0x{:X}", chain.base_offset),
                "offsets": chain.offsets,
                "resolved_address": format!("0x{:X}", resolved_addr)
            }),
            artifacts: vec![],
            limitations: vec![
                "Pointer validity depends on parent structure lifecycle".to_string(),
            ],
        })
    }

    pub fn write_preview(&self, request_id: &str, address: u64, expected_bytes: &[u8], new_bytes: &[u8]) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        let current_bytes = backend.read_memory(address, expected_bytes.len())?;
        let matches = current_bytes == expected_bytes;

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.write.preview".to_string(),
            status: if matches { "success" } else { "denied" }.to_string(),
            observations: json!({
                "address": format!("0x{:X}", address),
                "expected_bytes_hex": expected_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                "current_bytes_hex": current_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                "new_bytes_hex": new_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                "preimage_matches": matches,
                "restoration_plan": {
                    "rollback_bytes": current_bytes,
                    "target_address": format!("0x{:X}", address)
                }
            }),
            artifacts: vec![],
            limitations: vec![
                "Preview only: target memory was NOT modified".to_string(),
                "Explicit authorization required for write.commit".to_string(),
            ],
        })
    }

    pub fn write_commit(&self, request_id: &str, address: u64, expected_bytes: &[u8], new_bytes: &[u8], authorized: bool) -> Result<EvidenceEnvelope> {
        let b_guard = self.backend.lock().unwrap();
        let backend = b_guard.as_ref().ok_or(CheatError::NotConnected)?;
        let ident = self.target_identity.lock().unwrap().clone();

        if !authorized {
            return Ok(EvidenceEnvelope {
                request_id: request_id.to_string(),
                target_identity: ident,
                timestamp: current_timestamp(),
                operation: "memory.write.commit".to_string(),
                status: "denied".to_string(),
                observations: json!({
                    "error": "permission denied: policy authorization required; write commit rejected fail-closed"
                }),
                artifacts: vec![],
                limitations: vec![
                    "Unauthenticated write rejected fail-closed by policy gate".to_string(),
                ],
            });
        }

        // Preimage validation
        let current_bytes = backend.read_memory(address, expected_bytes.len())?;
        if current_bytes != expected_bytes {
            return Ok(EvidenceEnvelope {
                request_id: request_id.to_string(),
                target_identity: ident,
                timestamp: current_timestamp(),
                operation: "memory.write.commit".to_string(),
                status: "denied".to_string(),
                observations: json!({
                    "error": "preimage mismatch: target bytes do not match expected bytes",
                    "expected": expected_bytes,
                    "actual": current_bytes
                }),
                artifacts: vec![],
                limitations: vec![
                    "Write aborted due to concurrent modification or invalid expected state".to_string(),
                ],
            });
        }

        backend.authorize_write(true)?;
        // Always revoke elevated write permissions, including failed/partial writes.
        let write_result = backend.write_memory(address, new_bytes);
        let revoke_result = backend.authorize_write(false);
        revoke_result?;
        let written = write_result?;
        if written != new_bytes.len() {
            return Err(CheatError::InvalidArgument("partial memory write; success cannot be claimed".into()));
        }
        let post_bytes = backend.read_memory(address, new_bytes.len())?;
        if post_bytes != new_bytes {
            return Err(CheatError::InvalidArgument("memory readback mismatch after write".into()));
        }

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "memory.write.commit".to_string(),
            status: "success".to_string(),
            observations: json!({
                "address": format!("0x{:X}", address),
                "bytes_written": written,
                "preimage_hex": current_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                "postimage_hex": post_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
            }),
            artifacts: vec![
                format!("rollback:0x{:X}:{}", address, current_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("")),
            ],
            limitations: vec![
                "Mutation verified in active memory; game restart will restore default state".to_string(),
            ],
        })
    }

    pub fn verify_trainer(&self, request_id: &str, trainer_path: &str) -> Result<EvidenceEnvelope> {
        let ident = self.target_identity.lock().unwrap().clone();
        let path = std::path::Path::new(trainer_path);
        let exists = path.exists();

        Ok(EvidenceEnvelope {
            request_id: request_id.to_string(),
            target_identity: ident,
            timestamp: current_timestamp(),
            operation: "trainer.verify".to_string(),
            status: if exists { "success" } else { "error" }.to_string(),
            observations: json!({
                "trainer_path": trainer_path,
                "exists": exists,
                "is_file": path.is_file(),
            }),
            artifacts: vec![],
            limitations: vec![
                "Executable presence only; full behavioral verification requires controlled target execution".to_string(),
            ],
        })
    }
}
