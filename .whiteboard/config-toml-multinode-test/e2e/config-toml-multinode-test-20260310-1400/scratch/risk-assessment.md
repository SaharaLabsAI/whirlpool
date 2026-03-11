# Risk Assessment

## Iteration 1

### Resolved Risks
- **RISK-1: No existing config file support** → RESOLVED. Clean greenfield — no legacy config to break. We add `toml` + `serde` deps and a `--config` flag. NodeArgs already has all fields; we create a parallel `TomlConfig` struct.
- **RISK-2: Single-validator hardcode** → RESOLVED. main.rs line 50 needs to read validators from config instead of `vec![signer.public_key()]`. Straightforward change.

### Accepted Risks
- **RISK-3: In-process multi-node test complexity** → ACCEPTED. Running 4 nodes in-process with tokio tasks requires careful port allocation (use port 0 for OS-assigned) and tempdir isolation. Manageable with standard Rust test patterns.
- **RISK-4: Test timing sensitivity** → ACCEPTED. Block production depends on consensus timeouts (5s default). Test needs adequate timeout (60s+) and polling with backoff. Use shorter block intervals for tests.

### Blocker Conversions
- None.

### Expansion Summary
- No scope expansion needed.
