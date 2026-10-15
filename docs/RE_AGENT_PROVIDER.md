# RE-Agent integration decision

## Decision

**Use Cheat Agent as RE-Agent's memory-observation and authorized offline game-analysis provider; use GTLibCpp beneath Cheat Agent for typed memory sessions.** Do not embed a second independent Cheat Engine runtime in RE-Agent.

RE-Agent already contains a copy at [integrations/cheat-agent](https://github.com/heaven-hm/re-agent/tree/develop/integrations/cheat-agent), and RE-Agent has existing Frida/Ghidra static/runtime analysis, evidence, and bounded repair capabilities. A second memory scanner or independent LLM decision layer would duplicate functionality and increase audit cost.

Proposed layering:

```text
RE-Agent controller / MCP / evidence store
              |
       Cheat Agent provider
       |       |          |
   memory scan  AoB     trainer/verification evidence
       |
   GTLibCpp MemorySession + authorization (Windows native / adapters elsewhere)

Cheat Engine: optional external desktop debugging tool; do not copy/bundle code by default.
```

## First useful integration contract (proposed, not implemented)

Keep the provider behind an explicit, versioned protocol. Each operation returns an immutable evidence envelope with `request_id`, `target_identity`, `timestamp`, `operation`, `status`, `observations`, `artifacts`, and `limitations`.

| Operation | Purpose | Permission |
| --- | --- | --- |
| `target.inspect` | PID, executable path/hash, architecture, session status | read |
| `memory.regions` | Bounded region inventory and protections | read |
| `memory.scan.first` | Exact/unknown initial-value candidate scan | read |
| `memory.scan.next` | Narrow candidates using changed/unchanged/range criteria | read |
| `memory.aob` | Wildcard AoB matches in approved readable regions | read |
| `memory.pointer.resolve` | Architecture-aware pointer chain with provenance | read |
| `memory.read` | Typed bounded read with structured errors | read |
| `memory.write.preview` | Record expected bytes, target identity, and restoration plan | approval required |
| `memory.write.commit` | Scoped write with preimage check, auditing and rollback evidence | explicit authorization |
| `trainer.verify` | Report separately proven build, execution and behavior stages | controlled testing |

**Do not advertise these operations as implemented solely because this document defines them.** Existing Cheat Agent MCP commands must be mapped to this contract and verified end to end. Unimplemented calls must return explicit unsupported results.

## Safety and interoperability gates

- RE-Agent must bind each session to a verified executable identity (including architecture and launch instance), not a reusable PID alone.
- First/next scans need bounded data volumes, cancellation, time budgets, and persistent candidate provenance.
- Write calls must fail closed when authorization, expected current bytes or target identity do not match.
- A readback success is not proof of game behavior or of trainer survival after restart.
- The provider must be idempotent on request IDs where retrying could mutate state.
- Emit redacted, machine-readable diagnostics rather than unconstrained log strings.
- Test Windows x86/x64, Linux/macOS adapter paths and native/Rust FFI boundaries using controlled fixtures.
- RE-Agent must not silently use an outdated vendored snapshot. Prefer one pinned version and automated compatibility smoke tests.
- Separate licenses: Cheat Agent MIT does **not** relicense GTLibCpp's current Heaven-HM Community Modding License. Review redistribution before packaging binaries.

## Why not embed Cheat Engine directly?

Cheat Engine has rich scanning and debugging functionality, but the RE-Agent use case needs bounded machine-to-machine APIs, reproducible evidence, controlled permissions and clean integration. Its full application is too broad as the default provider. Use its public conceptual algorithms for comparisons; do not copy upstream implementation files without confirmed compatible licenses.

## Acceptance criteria for a follow-up implementation

1. Implement a real RE-Agent adapter with a schema-validated handshake, version compatibility and documented timeout/cancellation.
2. Wire read-only tool calls to actual Cheat Agent MCP/FFI operations and record evidence.
3. Add a separate opt-in and policy-checked path for mutation.
4. Add integration tests that detect missing features, invalid target identity, disconnection, partial reads and invalid architecture.
5. Verify that RE-Agent's vendored copy matches the pinned provider revision; document upgrade strategy.
6. Keep generated trainer behavior unverified until controlled execution proves it.
