# rpc-eth-reth-jsonrpc — Execution Plan

## TL;DR
- **Summary**: Replace `crates/rpc-eth`'s hand-rolled JSON-RPC stack with reth `EthApi` wiring through `WhirlpoolProvider`, `WhirlpoolTxPool`, and `WhirlpoolNetwork`, then update `whirlpool-node` and integration coverage to validate the new server path.
- **Deliverables**: reth-backed `rpc-eth` adapter modules, `RpcModuleBuilder` server wiring, `RpcConfig` public API, `whirlpool-node` integration, and end-to-end tests covering REQ-1..REQ-7 / TST-1..TST-12.
- **Effort**: 14 tasks, mostly sequential because provider trait coverage is the critical path; only late integration verification can overlap with final audit preparation.
- **Parallel**: Limited. Task 07 can proceed after Task 06, and final audit preparation can start once Tasks 12-13 are green.
- **Critical Path**: 01 -> 02 -> 03 -> 04 -> 05 -> 06 -> 07 -> 08 -> 09 -> 10 -> 11 -> 12 -> 13 -> 14.

## Context
- **Original Request**: Generate an execution plan under `.sisyphus/plans/rpc-eth-reth-jsonrpc/` from `.whiteboard/rpc-eth-reth-jsonrpc/`, following `agent/handoff.md`, REQ-1..REQ-7, and TST-1..TST-12 without touching implementation code.
- **Review Summary**: The design docs converge on an adapter-wrap strategy: `WhirlpoolProvider` bridges `state_reth::RethStateDb` into reth storage/provider traits, `WhirlpoolTxPool` bridges `app::TxSource`, `WhirlpoolNetwork` satisfies `reth_network_api::NetworkInfo`, and `server.rs` rewires startup through `reth_rpc_builder::RpcModuleBuilder`.
- **Resolution**: `agent/domains.md` still shows a legacy positional `start_rpc_server(...)` shape while `agent/crate-contracts/rpc-eth.md` and `agent/handoff.md` define `start_rpc_server(config: RpcConfig)`. This plan treats `RpcConfig` as canonical because it is the more specific contract and is required to remove the old `EthRpcContext` surface cleanly.
- **Resolution**: `agent/strategy.md` allows either zero-response or unsupported-response handling for `eth_blobBaseFee`, but REQ-5 / TST-10 require unsupported behavior. This plan standardizes on explicit unsupported-method behavior plus Type-3 rejection in `WhirlpoolTxPool`.

## Work Objectives
- **Core Objective**: Deliver a reth-backed `rpc-eth` server path that serves supported `eth_*` methods through adapter types instead of hand-rolled handlers.
- **Deliverables**:
  - `crates/rpc-eth/Cargo.toml` updated for reth RPC/provider/pool/network dependencies.
  - New `crates/rpc-eth/src/provider.rs`, `pool.rs`, `network.rs`, and `convert.rs` modules.
  - Rewritten `crates/rpc-eth/src/server.rs` and `lib.rs` around `RpcConfig` + `start_rpc_server`.
  - `crates/whirlpool-node/src/main.rs` and/or `crates/whirlpool-node/src/node.rs` wired to the new server API.
  - Integration coverage in `testing/integration-tests/tests/rpc_integration.rs` mirroring reth `rpc-builder` HTTP patterns.
- **Definition of Done**:
  - `nix develop --command cargo build -p rpc-eth`
  - `nix develop --command cargo test -p rpc-eth`
  - `nix develop --command cargo build -p whirlpool-node`
  - `nix develop --command cargo test -p whirlpool-node`
  - `nix develop --command cargo test -p integration-tests --test rpc_integration`
- **Must Have**:
  - Respect `agent/handoff.md` ordering and keep every task independently commit-ready.
  - Put behavior tests first within each implementation task.
  - Keep blob support excluded by design and never modify `vendor/**`.
  - Trace every task to `REQ-*` and `TST-*` ids.
- **Must NOT Have**:
  - No implementation work in `vendor/**`.
  - No cargo commands outside `nix develop --command ...` in gates.
  - No task larger than `L` or bundling multiple unrelated concerns.
  - No source-code edits performed as part of this planning session.

## Verification Strategy
- **ZERO HUMAN INTERVENTION**: Each committing task ends with explicit `nix develop --command cargo build/test ...` gates plus a task-specific behavioral check and an evidence write to `.sisyphus/evidence/task-NN-<slug>.md`.
- **Evidence Paths**:
  - `.sisyphus/evidence/task-01-rpc-eth-reth-dependencies.md`
  - `.sisyphus/evidence/task-02-provider-scaffold-and-stub-traits.md`
  - `.sisyphus/evidence/task-03-provider-block-and-header-readers.md`
  - `.sisyphus/evidence/task-04-provider-state-tx-and-receipt-readers.md`
  - `.sisyphus/evidence/task-05-provider-chain-context-and-subscriptions.md`
  - `.sisyphus/evidence/task-06-txpool-adapter-and-blob-rejection.md`
  - `.sisyphus/evidence/task-07-network-adapter.md`
  - `.sisyphus/evidence/task-08-conversion-layer.md`
  - `.sisyphus/evidence/task-09-rpcmodulebuilder-server-wiring.md`
  - `.sisyphus/evidence/task-10-public-api-and-legacy-surface-removal.md`
  - `.sisyphus/evidence/task-11-whirlpool-node-rpc-wiring.md`
  - `.sisyphus/evidence/task-12-basic-reth-rpc-integration-tests.md`
  - `.sisyphus/evidence/task-13-blob-and-remaining-rpc-integration-tests.md`
  - `.sisyphus/evidence/task-14-final-audit.md`

## Execution Strategy
### Parallel Execution Waves
- **Wave 1**: Task 01.
- **Wave 2**: Tasks 02-05 (sequential provider foundation slices).
- **Wave 3**: Tasks 06-08 (pool, network, conversion) after provider foundation.
- **Wave 4**: Tasks 09-10 (server wiring and public API cleanup).
- **Wave 5**: Tasks 11-13 (node wiring then end-to-end verification slices).
- **Wave 6**: Task 14 final audit.

### Dependency Matrix
- Task 01 -> Task 02
- Task 02 -> Task 03
- Task 03 -> Task 04
- Task 04 -> Task 05
- Task 05 -> Task 06, Task 07, Task 08
- Task 06 -> Task 09
- Task 07 -> Task 09
- Task 08 -> Task 09
- Task 09 -> Task 10
- Task 10 -> Task 11
- Task 11 -> Task 12
- Task 12 -> Task 13
- Task 13 -> Task 14

### Agent Dispatch Summary
- Provider-heavy tasks rely on `crates/rpc-eth/src/provider.rs` and should stay in one implementation lane to preserve trait-signature continuity.
- Integration tasks should reuse the existing `testing/integration-tests/tests/rpc_integration.rs` harness instead of inventing a second RPC test crate.
- The final audit task is `non-committing`; every earlier task must be commit-ready before Task 14 begins.

## Task List
<!-- TASKS_START -->
- [x] Task 01: Add reth RPC/provider dependencies [**S**] -> [tasks/01-rpc-eth-reth-dependencies.md](tasks/01-rpc-eth-reth-dependencies.md)
- [ ] Task 02: Scaffold `WhirlpoolProvider` and stub trait surface [**L**] -> [tasks/02-provider-scaffold-and-stub-traits.md](tasks/02-provider-scaffold-and-stub-traits.md)
- [ ] Task 03: Implement provider block/header/hash readers [**M**] -> [tasks/03-provider-block-and-header-readers.md](tasks/03-provider-block-and-header-readers.md)
- [ ] Task 04: Implement provider state, transaction, and receipt readers [**M**] -> [tasks/04-provider-state-tx-and-receipt-readers.md](tasks/04-provider-state-tx-and-receipt-readers.md)
- [ ] Task 05: Implement provider chain context and subscription adapters [**M**] -> [tasks/05-provider-chain-context-and-subscriptions.md](tasks/05-provider-chain-context-and-subscriptions.md)
- [ ] Task 06: Implement `WhirlpoolTxPool` and blob rejection [**M**] -> [tasks/06-txpool-adapter-and-blob-rejection.md](tasks/06-txpool-adapter-and-blob-rejection.md)
- [ ] Task 07: Implement `WhirlpoolNetwork` [**S**] -> [tasks/07-network-adapter.md](tasks/07-network-adapter.md)
- [ ] Task 08: Add `convert.rs` block and transaction conversion helpers [**M**] -> [tasks/08-conversion-layer.md](tasks/08-conversion-layer.md)
- [ ] Task 09: Rewire `server.rs` through `RpcModuleBuilder` [**M**] -> [tasks/09-rpcmodulebuilder-server-wiring.md](tasks/09-rpcmodulebuilder-server-wiring.md)
- [ ] Task 10: Rewrite `lib.rs` and remove legacy RPC surface [**S**] -> [tasks/10-public-api-and-legacy-surface-removal.md](tasks/10-public-api-and-legacy-surface-removal.md)
- [ ] Task 11: Update `whirlpool-node` for `RpcConfig` startup [**M**] -> [tasks/11-whirlpool-node-rpc-wiring.md](tasks/11-whirlpool-node-rpc-wiring.md)
- [ ] Task 12: Add basic reth-backed RPC integration coverage [**L**] -> [tasks/12-basic-reth-rpc-integration-tests.md](tasks/12-basic-reth-rpc-integration-tests.md)
- [ ] Task 13: Add blob exclusion and remaining RPC integration coverage [**M**] -> [tasks/13-blob-and-remaining-rpc-integration-tests.md](tasks/13-blob-and-remaining-rpc-integration-tests.md)
- [ ] Task 14: Final audit and evidence reconciliation [**S**] -> [tasks/14-final-audit.md](tasks/14-final-audit.md)
<!-- TASKS_END -->

## Artifact Registry
<!-- ARTIFACTS_START -->
| TestID | Planned Name | Actual Name | Location | Created By | Status |
|--------|--------------|-------------|----------|------------|--------|
| TST-1 | provider trait contract coverage | pending | `crates/rpc-eth/tests/provider_contract.rs` | Task 02 | pending |
| TST-2 | txpool adapter contract coverage | pending | `crates/rpc-eth/tests/pool_contract.rs` | Task 06 | pending |
| TST-3 | network adapter contract coverage | pending | `crates/rpc-eth/tests/network_contract.rs` | Task 07 | pending |
| TST-4 | RPC startup over HTTP | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 12 | pending |
| TST-5 | `eth_chainId` reth path coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 12 | pending |
| TST-6 | `eth_blockNumber` latest block coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 12 | pending |
| TST-7 | `eth_getBalance` state bridge coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 12 | pending |
| TST-8 | `eth_getBlockByNumber` block bridge coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 13 | pending |
| TST-9 | `eth_sendRawTransaction` tx submission coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 13 | pending |
| TST-10 | blob exclusion / unsupported method coverage | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 13 | pending |
| TST-11 | reth rpc-builder style permutations | pending | `testing/integration-tests/tests/rpc_integration.rs` | Task 13 | pending |
| TST-12 | `whirlpool-node` startup RPC smoke | pending | `crates/whirlpool-node/src/main.rs` and `testing/integration-tests/tests/rpc_integration.rs` | Task 11 | pending |
<!-- ARTIFACTS_END -->

## Final Verification
- Execute [tasks/14-final-audit.md](tasks/14-final-audit.md) after all committing tasks land; it reconciles the Artifact Registry, reruns the workspace verification set, and confirms REQ-1..REQ-7 / TST-1..TST-12 closure.
