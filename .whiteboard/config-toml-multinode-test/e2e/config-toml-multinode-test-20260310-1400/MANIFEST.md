# MANIFEST

## Inputs Consumed
- Cargo.toml (workspace structure)
- crates/whirlpool-node/src/config.rs (NodeArgs, NodeConfig, sub-configs)
- crates/whirlpool-node/src/main.rs (startup flow, validator hardcode)
- crates/whirlpool-node/Cargo.toml (dependencies)
- crates/consensus-simplex/src/config.rs (CommonwareConfig, validators field)
- crates/consensus-simplex/src/engine.rs (engine startup, validator Set)
- testing/integration-tests/ (RPC-only tests, no multi-node)
- docker/ (empty)
- agent-docs/index.md (project map)

## Outputs Produced
- .whiteboard/config-toml-multinode-test/agent/requirements.md
- .whiteboard/config-toml-multinode-test/e2e/config-toml-multinode-test-20260310-1400/e2e-state.md
- .whiteboard/config-toml-multinode-test/e2e/config-toml-multinode-test-20260310-1400/SKILL_DIGEST.md
- .whiteboard/config-toml-multinode-test/e2e/config-toml-multinode-test-20260310-1400/STATE_DELTA.md
- .whiteboard/config-toml-multinode-test/e2e/config-toml-multinode-test-20260310-1400/MANIFEST.md
