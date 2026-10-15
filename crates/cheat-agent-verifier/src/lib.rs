use cheat_agent_core::definition::CheatDefinition;
use cheat_agent_core::report::{VerificationReport, VerificationStages};
use cheat_agent_core::{CheatError, Result};
use cheat_agent_ffi::ProcessBackend;
use cheat_agent_generators::TrainerGenerator;
use std::process::{Child, Command};
use std::time::Duration;
use std::thread;

pub struct VerificationEngine;

impl VerificationEngine {
    pub fn verify_static(
        def: &mut CheatDefinition,
        target_pid: u32,
    ) -> Result<VerificationReport> {
        let mut report = VerificationReport {
            cheat_name: def.name.clone(),
            target_pid,
            overall_status: "UNVERIFIED".to_string(),
            stages: VerificationStages::default(),
            restart_runs: 0,
            evidence_log: Vec::new(),
        };

        let backend = ProcessBackend::new(target_pid)?;
        if !backend.is_alive() {
            return Err(CheatError::NotConnected);
        }

        report.evidence_log.push(format!("Attached to target PID: {}", target_pid));

        // 1. Resolve Address
        let resolved_addr = if def.resolution.method == "module_pointer_chain" {
            let chain = cheat_agent_core::PointerChain {
                base_module: def.resolution.module.clone().unwrap_or_default(),
                base_offset: u64::from_str_radix(
                    def.resolution.base_offset.as_deref().unwrap_or("0").trim_start_matches("0x"),
                    16,
                ).unwrap_or(0),
                offsets: def.resolution.offsets.as_ref().map_or(vec![], |offs| {
                    offs.iter()
                        .map(|s| i64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0))
                        .collect()
                }),
            };
            backend.resolve_pointer(&chain)?
        } else {
            u64::from_str_radix(
                def.resolution.static_address.as_deref().unwrap_or("0").trim_start_matches("0x"),
                16,
            ).unwrap_or(0)
        };

        report.evidence_log.push(format!("Resolved target address: 0x{:X}", resolved_addr));

        // 2. Read and Verify Memory
        let initial_bytes = backend.read_memory(resolved_addr, 4)?;
        report.evidence_log.push(format!("Original memory bytes: {:02X?}", initial_bytes));
        report.evidence_log.push("Read-only observation: no target write was attempted or verified".to_string());

        // 3. Test Trainer Code Generation & Compilation
        let cpp_code = TrainerGenerator::generate_cpp(def)?;
        let temp_dir = std::env::temp_dir();
        let src_path = temp_dir.join("trainer_test.cpp");
        let bin_path = temp_dir.join("trainer_test_bin");

        std::fs::write(&src_path, &cpp_code)
            .map_err(|e| CheatError::Internal(format!("Failed to write temp trainer: {}", e)))?;

        // Attempt compilation with system compiler
        let compile_status = Command::new("clang++")
            .arg("-std=c++20")
            .arg(&src_path)
            .arg("-o")
            .arg(&bin_path)
            .status();

        if let Ok(st) = compile_status {
            if st.success() {
                report.stages.trainer_compilation = true;
                report.evidence_log.push("Generated C++20 trainer compiled successfully with 0 errors".to_string());
            } else {
                report.evidence_log.push("Compiler exited with non-zero code; check C++20 toolchain".to_string());
            }
        } else {
            report.evidence_log.push("clang++ compiler not detected in test environment".to_string());
        }

        report.evidence_log.push("Behavior and trainer execution were not tested; verification remains incomplete".to_string());
        def.verification.status = "unverified".to_string();
        def.verification.evidence = report.evidence_log.clone();

        Ok(report)
    }

    pub fn verify_with_restart(
        def: &mut CheatDefinition,
        target_command: &str,
    ) -> Result<VerificationReport> {
        let mut report = VerificationReport {
            cheat_name: def.name.clone(),
            target_pid: 0,
            overall_status: "UNVERIFIED".to_string(),
            stages: VerificationStages::default(),
            restart_runs: 0,
            evidence_log: Vec::new(),
        };

        // Launch target process run 1
        let mut child1 = spawn_process(target_command)?;
        let pid1 = child1.id();
        report.target_pid = pid1;
        report.evidence_log.push(format!("Launch 1: started target game PID {}", pid1));
        thread::sleep(Duration::from_millis(100));

        let backend1 = ProcessBackend::new(pid1)?;
        if !backend1.is_alive() {
            return Err(CheatError::NotConnected);
        }
        report.evidence_log.push("Process was alive; no memory write or game behavior was validated".to_string());

        // Terminate run 1
        let _ = child1.kill();
        let _ = child1.wait();
        report.evidence_log.push(format!("Launch 1: terminated PID {}", pid1));
        thread::sleep(Duration::from_millis(100));

        // Launch target process run 2 (Restart verification)
        let mut child2 = spawn_process(target_command)?;
        let pid2 = child2.id();
        report.restart_runs = 1;
        report.evidence_log.push(format!("Launch 2 (Restart): started target game PID {}", pid2));
        thread::sleep(Duration::from_millis(100));

        let backend2 = ProcessBackend::new(pid2)?;
        if backend2.is_alive() {
            report.evidence_log.push("Process restarted, but cheat address persistence and trainer behavior remain unverified".to_string());
        }

        let _ = child2.kill();
        let _ = child2.wait();

        report.evidence_log.push("No cheat write, trainer compilation, or trainer execution was performed in restart test".to_string());

        def.verification.status = "unverified".to_string();
        def.verification.evidence = report.evidence_log.clone();

        Ok(report)
    }
}

fn spawn_process(cmd: &str) -> Result<Child> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(CheatError::InvalidArgument("empty target command".to_string()));
    }
    let mut command = Command::new(parts[0]);
    if parts.len() > 1 {
        command.args(&parts[1..]);
    }
    command.spawn().map_err(|e| CheatError::Internal(format!("Failed to spawn {}: {}", cmd, e)))
}
