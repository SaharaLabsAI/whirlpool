# Handoff

## Design → Plan Handoff

### Implementation Order (strict)
1. **Config TOML support** (whirlpool-node/config.rs)
   - Add toml + serde deps to Cargo.toml
   - Create TomlConfig struct with Deserialize
   - Add --config and --validator CLI args to NodeArgs
   - Add validators field to NodeConfig
   - Implement load_config() with merge logic
   - Unit tests: TOML loading, merge precedence, validator parsing

2. **Multi-validator wiring** (whirlpool-node/main.rs)
   - Replace hardcoded `vec![signer.public_key()]` with config.validators
   - Fallback to single-signer if no validators specified

3. **Node startup extraction** (whirlpool-node/main.rs → lib.rs)
   - Extract start_node(NodeConfig) → NodeHandle
   - NodeHandle: rpc_addr, shutdown signal, JoinHandle
   - main() becomes thin wrapper: parse → load_config → start_node → wait

4. **4-node integration test** (testing/integration-tests/)
   - Add whirlpool-node + commonware-cryptography deps
   - Create tests/multinode_consensus.rs
   - Implement test_four_node_consensus() per Flow 2

### Key Constraints
- All cargo commands via `nix develop --command <cmd>`
- Behavior tests before implementation
- No vendor/ modifications
- Update agent-docs after code changes

### Test Verification
- `cargo test -p whirlpool-node` — config unit tests pass
- `cargo test -p integration-tests` — multinode test passes
- `cargo build` — full workspace builds clean
