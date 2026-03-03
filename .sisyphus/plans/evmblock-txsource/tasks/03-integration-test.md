# Task 03: Integration test

## Summary
Implement a high-level integration test that verifies the full push-propose flow. This test ensures that when a transaction is pushed to the pool, it is correctly included in the next block proposed by the application.

## Crate(s)
- `app-evm` (primary)

## Files Changed
- `crates/app-evm/tests/integration.rs` — Addition of the integration test.

## Dependencies
- Task 01: InMemoryTxPool implementation + unit tests
- Task 02: Node wiring update

## Design Refs
- `docs/design/evmblock-txsource/FLOWS.md S-4`
- `docs/design/evmblock-txsource/TESTS.md T-7`

## TDD Sequence
1. **Red**: Add `test_propose_with_in_memory_pool` to `integration.rs` and verify it fails (either compilation or execution).
2. **Green**: Update the test to push a transaction to the pool and call `app.propose()`, asserting that the resulting block contains the transaction.
3. **Verify**: Run AC commands to ensure the integration test passes.

## Implementation Details
The test should instantiate an `EvmApplication` with an `InMemoryTxPool`. It should push a raw transaction byte vector to the pool, call `propose()` on the application, and verify that the returned `ProposedBlock` includes the pushed transaction in its body.

## Acceptance Criteria
```
nix develop --command cargo test -p app-evm --test integration test_propose_with_in_memory_pool
```

## Evidence
- `.sisyphus/evidence/evmblock-txsource/03-integration-test.log`
