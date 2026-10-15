# Multi-Agent PR Review Protocol & Quality Gates

This document defines the automated multi-agent adversarial review gates, security standards, and release criteria for `cheat-agent`.

---

## 1. Core Principles

1. **Adversarial Roles Split:**
   - **Critique Specialist (Native C++ & Rust FFI Auditor):** Audits pointer dereferences, chunk boundary crossings, C ABI buffer allocations, process handles, and memory map arithmetic.
   - **Validation Specialist (Verification Engineer):** Runs automated test games, validates that incomplete verification stages remain `UNVERIFIED` fail-closed, and enforces cross-platform CI passes.
   - **Release Gatekeeper:** Enforces policy sign-off: minimum review depth, deduplicated finding logs, and mandatory human sign-off before default branch merges.

2. **Evidence Before Assertions:**
   - Every critique must include a concrete failure scenario or reproduction test.
   - Verification stages must never claim success (`PASSED`, `restart_verified`) without executing the underlying live verification operations.

3. **Deterministic CI Supremacy:**
   - AI review is advisory. Deterministic CI build and test matrix passes across Linux x64, Windows x64, and macOS Apple Silicon are required hard gates.

---

## 2. Review Dimensions & Checklist

### A. Memory Safety & Native C/C++ Engine
- [ ] Scan buffer widths validated before copying (`sizeof(T) <= value_size`).
- [ ] Scanning across 64 KiB chunk boundaries preserves pattern and typed match continuity.
- [ ] Pattern lengths capped (e.g. 4 KiB limit) to prevent unbounded memory usage or algorithmic stalls.
- [ ] Memory region arithmetic guards against integer overflow.

### B. Rust FFI & Agent Tool Protocol
- [ ] Model tool dispatch allowlists tools and rejects unrecognized tool names fail-closed.
- [ ] Standalone agent loop enforces strict iteration ceiling (`--max-iterations`) and tool budgets.
- [ ] Unimplemented or unverified tools do not manufacture synthetic pass reports.
- [ ] Agent-facing tools maintain read-only inspection boundaries unless explicit policy authorization is attached.

### C. GTLibCpp Backend Integration
- [ ] GTLibCpp adapter enforces 1 MiB safety caps and detects address space wrapping.
- [ ] Architecture-aware pointer resolution correctly distinguishes 32-bit (x86) and 64-bit (x64) pointer widths.
- [ ] Unauthenticated writes or freeze mutations return `CHEAT_STATUS_PERMISSION_DENIED`.

### D. Verification Pipeline
- [ ] Negative verification cases (e.g. invalid addresses, stopped targets) tested and fail closed.
- [ ] All unit and integration tests run and pass against the exact PR head commit.

---

## 3. Secure Environment Configuration

To run review agents via Claude Code, OpenAI Codex, or direct API providers:
1. Store API keys strictly in environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY`, `OLLAMA_HOST`).
2. Never commit secrets, session tokens, or sensitive credentials into repository files or test configurations.
3. Automated push and merge rights to `main` must be disabled for bot tokens; merges require human review and approval.
