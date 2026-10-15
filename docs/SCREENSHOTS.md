# Screenshot capture and verification

Use real, reproducible captures from an authorized offline test target. Do not present mock console text as an executed result.

1. Build native tests with `cmake -S . -B build && cmake --build build` and capture `ctest --test-dir build --output-on-failure`.
2. Run `cargo test --workspace`, then capture a terminal showing the passing test summary.
3. Launch `examples/test-game`, run an authorized `cheat-agent scan --pid <PID> --exact 100 --data-type uint32`, and capture matching addresses with process identity visible.
4. Capture `cheat-agent mcp` inside an MCP-capable host showing actual returned tool data.
5. Capture `cheat-agent run` invoking a real scan tool through Ollama or OpenAI. Label model, platform, program hash, invocation and whether the goal was independently verified.
6. Capture generated Cheat Engine `.CT` output in a text editor; only depict a running Cheat Engine application if independently installed and exercised.

Store redacted authentic images under `docs/images/` as `native-tests.png`, `scan.png`, `mcp.png`, `standalone-agent.png`, and `cheat-engine-table.png`. Strip API keys, private paths and unrelated process details. Add screenshot links to the README **only after these actual captures are committed**.
