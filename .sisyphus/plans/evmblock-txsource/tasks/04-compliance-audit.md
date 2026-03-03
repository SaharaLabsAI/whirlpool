# Task 04: Full compliance audit

## Summary
Perform a final workspace-wide audit to ensure the `InMemoryTxPool` implementation and node wiring are consistent with the design and that all tests pass. This task also includes updating the codebase documentation.

## Crate(s)
- `app`
- `whirlpool-node`
- `app-evm`

## Files Changed
- None (verification task)

## Dependencies
- Task 01: InMemoryTxPool implementation + unit tests
- Task 02: Node wiring update
- Task 03: Integration test

## Design Refs
- `docs/design/evmblock-txsource/INTENT.md SC-1 through SC-7`

## TDD Sequence
1. **Red**: N/A
2. **Green**: N/A
3. **Verify**: Run full workspace build and test suite.

## Implementation Details
Verify that the `InMemoryTxPool` correctly implements the `TxSource` trait and that the node wiring is properly injecting it. Confirm that the integration test passes alongside existing unit tests. Check for any documentation updates required for the new components.

## Acceptance Criteria
```
nix develop --command cargo build
nix develop --command cargo test -p app
nix develop --command cargo test -p app-evm
nix develop --command cargo test
```

## Evidence
- `.sisyphus/evidence/evmblock-txsource/04-compliance-audit.log`
