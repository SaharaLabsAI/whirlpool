# Task 09: Rewire `server.rs` through `RpcModuleBuilder`

## Status
- pending

## Dependencies
- 06
- 07
- 08

## Wave
- Wave 4

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Tasks 06, 07, and 08 are complete and committed.
- [ ] Provider, pool, network, and conversion modules compile together.
- [ ] TST-4, TST-5, TST-6, and TST-10 remain pending for later integration coverage.
- [ ] Scope is limited to `crates/rpc-eth/src/server.rs` and closely related internal exports.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/strategy.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crate-contracts/rpc-eth.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `crates/rpc-eth/src/server.rs`
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/src/pool.rs`
  - `crates/rpc-eth/src/network.rs`
  - `crates/rpc-eth/src/convert.rs`
- Vendor references:
  - `vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs`

## Vendor Usage Patterns
- Follow the reth HTTP startup pattern from `vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs` for transport config and module construction.
- Install the blob unsupported behavior at the server/module layer without editing vendor code.

## What to do
1. Add server-focused executable coverage first in `crates/rpc-eth/tests/server_contract.rs` or the existing crate test harness so startup can be built around TST-4-style expectations before integration tests land.
2. Rewrite `crates/rpc-eth/src/server.rs` to construct `WhirlpoolProvider`, `WhirlpoolTxPool`, and `WhirlpoolNetwork`, then pass them into `RpcModuleBuilder` with the required EVM and consensus objects.
3. Bootstrap the reth Eth API and configure the HTTP transport using the local bind address.
4. Install explicit unsupported handling for `eth_blobBaseFee` if the default provider behavior alone does not satisfy REQ-5.
5. Remove legacy `EthApiHandler`/manual JSON-RPC startup usage from this module while keeping the public entrypoint name stable for Task 10.

## Mock Boundary
- Use minimal in-crate startup tests or fakes for builder contract checks.
- Do not add end-to-end HTTP assertions here; those belong to Tasks 12-13.

## AC trace
- REQ-1
- REQ-5
- TST-4
- TST-5
- TST-6
- TST-10

## Must NOT do
- Do not finalize the public `RpcConfig` API in `lib.rs` yet.
- Do not modify `crates/whirlpool-node/**`.
- Do not add the main integration test assertions in this task.

## Acceptance Criteria
- [ ] `crates/rpc-eth/src/server.rs` uses `RpcModuleBuilder` and the new adapter types.
- [ ] Legacy manual RPC server wiring is removed from `server.rs`.
- [ ] Blob unsupported behavior is explicitly handled.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `.sisyphus/evidence/task-09-rpcmodulebuilder-server-wiring.md` records commands and outcomes.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth server_contract`
- [ ] Evidence file notes the exact unsupported-method handling chosen for `eth_blobBaseFee`.
- [ ] Artifact Registry keeps TST-4/TST-5/TST-6/TST-10 integration rows pending.
- [ ] Create one dedicated git commit for this task before starting Task 10.

## Post-Task Reconciliation
- If startup contract tests were added, record their names in evidence for later registry updates.

## QA Scenarios
- Happy path: builder-based server startup compiles and returns a server handle.
- Failure path: missing transport config or unsupported-method override causes a failing startup test.
- Boundary case: blob method path is intercepted without disabling other `eth_*` methods.

## Evidence
- `.sisyphus/evidence/task-09-rpcmodulebuilder-server-wiring.md`
