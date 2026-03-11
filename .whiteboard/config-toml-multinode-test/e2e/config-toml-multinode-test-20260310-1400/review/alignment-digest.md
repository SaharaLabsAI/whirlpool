# Alignment Digest

## Approved Intent
Add config.toml file support to whirlpool-node and create a 4-node in-process integration test that verifies P2P connectivity via block height growth.

## Confirmed Scope
1. **Config TOML support**: Add `--config <path>` CLI flag, `toml`+`serde` deps, `TomlConfig` struct with deserialization, CLI-over-TOML-over-defaults layering
2. **Multi-validator config**: Add `validators` field (Vec<hex pubkey>) to config, replace hardcoded single-validator in main.rs
3. **4-node integration test**: In-process tokio tasks, tempdir isolation, OS-assigned ports, shared validator set, bootstrap peer wiring
4. **Block height verification**: Poll block height across all nodes, assert growth within timeout

## Approach Direction
- Extend existing `NodeArgs`/`NodeConfig` in config.rs (not replace)
- Add parallel `TomlConfig` struct with serde Deserialize
- Merge logic: load TOML first if --config provided, then overlay CLI args
- Integration test in `testing/integration-tests/` using actual whirlpool-node startup code (not a new binary)
- Use short block intervals (1s) in tests for fast feedback

## Risks
- Test timing sensitivity (accepted — use adequate timeouts + polling)
- In-process multi-node complexity (accepted — standard tokio task pattern)

## Iteration Count
1
