## Change scope
- Linked issue:
- Why this change is needed:
- Platforms affected: Windows / Linux / macOS
- C ABI / Rust FFI contract change (yes/no):
- Process memory access or authorization change (yes/no):

## Independent review evidence
- [ ] C/C++ reviewer: bounds, pointer arithmetic, ownership, process lifetime, and native memory correctness
- [ ] Rust/FFI reviewer: lifetimes, panic/exception boundaries, C ABI type/length contracts
- [ ] Security reviewer: target identity, read-only default, explicit mutation authorization, secrets and API safety
- [ ] Behavior reviewer: no fabricated verification; negative-path tests and real target/trainer behavior
- [ ] Portability reviewer: Windows, Linux, macOS behavior and platform limitations
- [ ] Regressions: test for every confirmed P0/P1 fix
- [ ] Exact current head SHA tested across the applicable CI matrix
- [ ] All actionable review comments addressed with evidence

## Validation at latest commit
| Gate | Command / CI link | Result |
| --- | --- | --- |
| CMake build + CTest | | |
| Cargo workspace tests | | |
| Rust FFI / ABI contract checks | | |
| Negative verifier tests | | |
| Windows / Linux / macOS CI | | |

## Merge policy
Do not merge while any P0/P1 finding, required CI job, or independent human approval remains unresolved. AI and bot review are advisory, not independent human approval. Never interpret a process restart or code generation as evidence of trainer execution without actually testing the behavior. This template does not itself enforce branch protection.
