# Task 05: Wire workspace and dual RPC node composition

**Complexity**: M

## Summary
Finish workspace membership and node composition so `whirlpool-node` owns shared `TxSource`, dual RPC startup, and the prototype personality store.

## Requirements
- REQ-1
- REQ-2
- REQ-3
- REQ-7

## Tests
- TST-001
- TST-005

## Mock Boundary
Prefer node wiring tests and focused startup checks; avoid end-to-end network orchestration beyond what is needed to verify dual RPC composition.

## What to do
1. Add tests or startup checks that prove `rpc-eth` and `rpc-mem` are both wired from `whirlpool-node` without merging responsibilities.
2. Add `crates/app-mem` and `crates/rpc-mem` to `Cargo.toml` workspace membership and wire dependencies.
3. Update `crates/whirlpool-node/src/node.rs` to construct the mem store, start `rpc-mem`, and share chain/mempool dependencies across both RPC servers.
4. Preserve `rpc-eth` behavior and the generic opaque-byte mempool contract.
5. Verify the affected workspace crates compile together.

## Acceptance Criteria
```bash
nix develop --command cargo build -p whirlpool-node -p rpc-mem -p app-mem
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-05-node-wiring.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
