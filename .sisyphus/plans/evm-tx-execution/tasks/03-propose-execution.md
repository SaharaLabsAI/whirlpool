# Task 03: propose-execution

## Summary
Replace the `propose()` stub in `app-evm` with full EVM transaction execution using reth's block builder.

## Crate(s)
`app-evm`

## Files Changed
`crates/app-evm/src/executor.rs`, `crates/app-evm/Cargo.toml`

## Dependencies
Task 01 (state-commit-tests), Task 02 (tx-decode-helper)

## Design Refs
`FLOWS.md §F1`, `TESTS.md T-1, T-2, T-3, T-7`

## TDD Sequence
1. Write T-7: `propose_empty_txsource_produces_empty_block` (Red)
2. Implement basic proposal loop — empty tx source returns empty block (Green for T-7)
3. Write T-1: `propose_executes_transfer_transaction` (Red)
4. Implement full EVM execution flow — tx decode, builder, execute, commit, compute roots (Green for T-1)
5. Write T-2: `propose_executes_contract_deployment` (Red)
6. Verify T-2 passes with existing implementation (Green for T-2, no new production code expected)
7. Write T-3: `propose_skips_invalid_transactions` (Red)
8. If T-3 fails, add skip-on-failure logic to execution loop (Green for T-3)

## Implementation Details
1. Fetch pending transactions from `tx_source.pending()`.
2. Decode transactions via `decode_transactions` helper (skip failures).
3. Clone `state_db` for a **snapshot** (execution runs against this clone, NOT canonical).
4. Wrap the **cloned snapshot** in `reth_revm::State`.
5. Use `evm_config.builder_for_next_block` with `NextBlockEnvAttributes`.
6. For each transaction, use `builder.execute_transaction(tx)` (skip failures).
7. Call `state.take_bundle()` after execution.
8. Call `canonical_db.commit(&bundle)` to update the canonical state.
9. Compute `tx_root` and `receipts_root` using `alloy-trie::ordered_trie_root_with_encoder`.
10. Return an `EvmBlock` with computed fields.

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm -- propose` passes
- `nix develop --command cargo build -p app-evm` succeeds
- No new warnings

## Evidence
- Path: `.sisyphus/evidence/evm-tx-execution/03-propose-execution.log`
