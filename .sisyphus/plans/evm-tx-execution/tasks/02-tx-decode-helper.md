# Task 02: tx-decode-helper

## Summary
Implement a helper function to decode raw transaction bytes into recovered Ethereum transactions.

## Crate(s)
`app-evm`

## Files Changed
`crates/app-evm/src/executor.rs`, `crates/app-evm/Cargo.toml`

## Dependencies
None

## Design Refs
`FLOWS.md §F1 step 2`, `CRATES.md S-1`

## TDD Sequence
1. Write unit test for `decode_transactions` with valid RLP bytes (Red)
2. Write unit test for `decode_transactions` with invalid RLP bytes (Red)
3. Write unit test for `decode_transactions` with empty input (Red)
4. Implement `decode_transactions` helper (Green)

## Implementation Details
1. Add `reth-ethereum-primitives` and `reth-primitives-traits` to `Cargo.toml` if not already present as direct dependencies.
2. Implement `decode_transactions(&[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError>` in `crates/app-evm/src/executor.rs`.
3. Use `TransactionSigned::decode_2718()` for RLP decoding.
4. Use `try_recover()` on the decoded transaction to obtain `RecoveredTx` (with sender recovery).

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm -- decode_transactions` passes
- `nix develop --command cargo build -p app-evm` succeeds
- No new warnings

## Evidence
- Path: `.sisyphus/evidence/evm-tx-execution/02-tx-decode-helper.log`
