# Config TOML + Multi-Node Test Plan

**TL;DR**: Add config.toml file support to whirlpool-node with CLI > TOML > defaults layering, multi-validator configuration, extract start_node() for testability, and create a 4-node in-process integration test verifying P2P connectivity via block height growth and peer connection logging.

## Context
- Workspace: `/home/dev/sahara/web3/agent/playground/whirlpool`
- Design docs: `.whiteboard/config-toml-multinode-test/`
- Primary crate: `crates/whirlpool-node/` (config.rs, main.rs, lib.rs, Cargo.toml)
- Test crate: `testing/integration-tests/` (new test file)
- Build command: `nix develop --command cargo build`
- Test command: `nix develop --command cargo test`

## Objectives
1. Support config.toml file loading via `--config <path>` CLI flag (REQ-1)
2. CLI > TOML > defaults config layering (REQ-2)
3. Multi-validator support from config (REQ-3)
4. 4-node in-process integration test with P2P connectivity proof (REQ-4, REQ-6)
5. Block height growth verification across all 4 nodes (REQ-5)
6. Backward compatibility preserved (NFR-1)

## Verification
- `nix develop --command cargo build` — workspace builds clean
- `nix develop --command cargo test -p whirlpool-node` — config unit tests pass
- `nix develop --command cargo test -p integration-tests -- --test-threads=1 multinode` — multi-node test passes

## Execution Strategy
Tasks execute in strict order. Each task is behavior-test-first: write tests that define expected behavior, then implement to make them pass.

<!-- TASKS_START -->
- [ ] Task 1: Config TOML loading and CLI layering [**M**] -> [tasks/01-config-toml-loading.md]
- [ ] Task 2: Multi-validator configuration [**M**] -> [tasks/02-multi-validator-config.md]
- [ ] Task 3: Extract start_node() for testability [**M**] -> [tasks/03-extract-start-node.md]
- [ ] Task 4: 4-node integration test with P2P + height verification [**L**] -> [tasks/04-multinode-integration-test.md]
<!-- TASKS_END -->

## Artifact Registry

| TestID | AC/INV | File | Description |
|--------|--------|------|-------------|
| TST-01 | AC-1 | whirlpool-node/src/config.rs | TOML file loading |
| TST-02 | AC-2 | whirlpool-node/src/config.rs | CLI overrides TOML |
| TST-03 | AC-3 | whirlpool-node/src/config.rs | Backward compat (no --config) |
| TST-04 | AC-4 | whirlpool-node/src/config.rs | Multi-validator from config |
| TST-05 | AC-5 | integration-tests/tests/multinode_consensus.rs | 4-node P2P connectivity |
| TST-06 | AC-6 | integration-tests/tests/multinode_consensus.rs | Block height growth all 4 |
| TST-07 | AC-7 | integration-tests/tests/multinode_consensus.rs | Per-node height+peer logging |
| TST-08 | QA-1 | whirlpool-node/src/config.rs | Missing config file error |
| TST-09 | QA-2 | whirlpool-node/src/config.rs | Invalid TOML error |
| TST-10 | QA-3 | whirlpool-node/src/config.rs | Partial TOML + CLI merge |
| TST-11 | QA-4 | whirlpool-node/src/config.rs | Empty validators rejection |

## Final Verification
```bash
nix develop --command cargo build 2>&1 | tail -5
nix develop --command cargo test 2>&1 | tail -20
nix develop --command cargo clippy --all-targets 2>&1 | tail -10
```
