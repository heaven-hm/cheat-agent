# Memory backend decision: GTLibCpp vs Cheat Engine

## Decision

Use **GTLibCpp** for typed, bounded target memory reads/writes, pointer-chain operations, and local authorization interfaces. Keep Cheat Agent's C++ scanner, native OS process backends and Rust CLI/MCP/model layer as separate components. Treat the Cheat Engine project as an **algorithm reference** for first/next scans, candidate filtering, alignments and protection-aware memory enumeration, rather than importing its Pascal implementations.

This architecture gives the application one explicit memory-session interface without coupling the AI workflow to an entire debugger GUI/runtime. GTLibCpp's native Win32 backend can service Windows clients; on Linux/macOS the existing Cheat Agent backends can be wrapped in the `GtlibMemoryBackendAdapter`. The latter is not the same as a GTLibCpp native Linux/macOS process implementation.

## Build and reproducibility

The native CMake integration pins GTLibCpp to commit
`71ed9a5d5ab502ae198a9808465ed93016fcb6e6` from its `develop` line,
rather than following its older `master` branch. A sibling checkout
`../gtlibcpp` is still preferred when present and may be at another revision,
so verify that revision during release packaging.

```sh
cmake -S . -B build -DCHEAT_AGENT_ENABLE_GTLIBCPP=ON
cmake --build build --config Release
ctest --test-dir build -C Release --output-on-failure

# Build the core without the optional GTLibCpp adapter:
cmake -S . -B build-no-gtlib -DCHEAT_AGENT_ENABLE_GTLIBCPP=OFF
cmake --build build-no-gtlib --config Release
ctest --test-dir build-no-gtlib -C Release --output-on-failure
```

The CI workflow checks both configurations. The optional Project IGI trainer
uses GTLibCpp's Windows backend and requires the dependency enabled. **This
does not establish that every Rust CLI, MCP command, or generated trainer
uses GTLibCpp.** Those pathways need separate wiring and end-to-end tests.

## Licensing is a release gate

Cheat Agent advertises **MIT**, but GTLibCpp's current distribution is covered by
the custom **Heaven-HM Community Modding License**, with noncommercial and
redistribution restrictions. These are not equivalent licenses. Do not label
a distribution containing GTLibCpp source or binaries as unconditionally
MIT-licensed; preserve the separate GTLibCpp license and attribution, and
obtain an explicit compatible grant if commercial or MIT-only redistribution
is intended.

Cheat Engine's upstream source also must not be copied/bundled without
checking the license applicable to each file and resulting redistribution.
The original scanner implementations in Cheat Agent remain their own code.

## Verification priorities

- Confirm that all mutation entry points enforce intended authorization and
  target identity, not merely a caller-set Boolean.
- Regression test x86/x64 pointer sizes, boundary addresses and partial reads.
- Validate scanner state transitions (first/next scan) with controlled fixtures.
- Validate actual trainer behavior across process restarts before marking a
  cheat fully verified.
- Test library integration on Windows x86 and x64 and adapter behavior on
  Linux/macOS. Run independent security/ABI reviews before shipping.
