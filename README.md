# Cheat Agent

Experimental C++20/Rust memory-analysis and trainer-generation tools for **authorized offline games and test processes**.

[CI](https://github.com/heaven-hm/cheat-agent/actions/workflows/ci.yml) · [Examples](docs/EXAMPLES.md) · [Architecture](docs/ARCHITECTURE.md) · [Security](docs/SECURITY.md) · [License](LICENSE)

## Two ways to use it

| Mode | Entry point | Current capability |
| --- | --- | --- |
| **Standalone agent** | `cheat-agent run` | Model-driven iterative calls into native scan, inspection and generation tools |
| **Hosted coding agent** | `cheat-agent mcp` | MCP tools callable from Codex, Claude Code or another compatible host |
| **Direct CLI** | `cheat-agent scan`, `generate`, `verify` | Manual memory scans, trainer generation and experimental verification |

The native scanner supports exact/AoB scanning, changed/increased/decreased refinement and pointer-chain search through its MCP interface. The generator emits C++, Rust and Cheat Engine `.CT` tables. **It does not embed or control the Cheat Engine GUI.** The five-stage real-game verification pipeline remains incomplete.

## Build and test

```bash
cmake -S . -B build -DCHEAT_AGENT_ENABLE_GTLIBCPP=ON
cmake --build build --config Release
ctest --test-dir build --output-on-failure
cargo build --workspace
cargo test --workspace
```

Requires CMake 3.20+, C17/C++20 compiler, Rust toolchain and access to the pinned GTLibCpp source. See the [full build guide](docs/BUILD.md).

## Standalone mode

```bash
cargo run -p cheat-agent-cli -- run --provider ollama --model qwen2.5-coder:32b --goal "Inspect my offline test process and find the changing health value"
```

## Hosted MCP mode

```json
{
  "mcpServers": {
    "cheat-agent": {
      "command": "cheat-agent",
      "args": ["mcp"]
    }
  }
}
```

## Direct scanning and trainer generation

```bash
cheat-agent scan --pid 1234 --exact 100 --data-type uint32
cheat-agent scan --pid 1234 --aob "48 8B 05 ?? ?? ?? ??"
cheat-agent generate --cheat examples/test-game/test_cheat.json --lang ct
```

`scan` currently accepts `uint32` exact values or AoB patterns. Use MCP for additional data types and scan refinement.

## Screenshots and demonstrations

Real terminal and tool demonstrations will be added after a reproducible build and recorded offline test run. No mock success images are presented as evidence. See [capture checklist](docs/SCREENSHOTS.md).

See [usage examples](docs/EXAMPLES.md), [cross-platform notes](docs/CROSS_PLATFORM.md), and [security](docs/SECURITY.md).