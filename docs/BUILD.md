# Build and test

## Requirements

- CMake 3.20+
- C17/C++20 compiler (MSVC, Clang or GCC)
- Stable Rust/Cargo toolchain
- GTLibCpp dependency reachable as a sibling checkout or via the pinned FetchContent Git commit

## Native build

```bash
cmake -S . -B build -DCHEAT_AGENT_ENABLE_GTLIBCPP=ON
cmake --build build --config Release
ctest --test-dir build --output-on-failure
```

GTLibCpp can be disabled for independent native diagnostics with `-DCHEAT_AGENT_ENABLE_GTLIBCPP=OFF`, but this removes its adapter coverage.

## Rust workspace

```bash
cargo build --workspace
cargo test --workspace
```

The Rust FFI crate may require built native libraries and toolchain configuration; consult any platform-specific CI workflow if linking fails. A successful build does not prove live game behavior, address persistence or trainer effectiveness.

## CLI examples

```bash
cargo run -p cheat-agent-cli -- scan --pid 1234 --exact 100 --data-type uint32
cargo run -p cheat-agent-cli -- generate --cheat examples/test-game/test_cheat.json --lang ct
cargo run -p cheat-agent-cli -- run --provider ollama --model qwen2.5-coder:32b --goal "Inspect my authorized test process"
```

Do not run against targets you do not own or have permission to inspect. For reproducible screenshot evidence see [SCREENSHOTS.md](SCREENSHOTS.md).
