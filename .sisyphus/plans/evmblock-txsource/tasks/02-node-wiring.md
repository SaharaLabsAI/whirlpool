# Task 02: Node wiring update

## Summary
Update the node binary to use the new `InMemoryTxPool`. This involves replacing the `NoopTxSource` with an `Arc<InMemoryTxPool>` and injecting it into the `EvmApplication` during initialization.

## Crate(s)
- `whirlpool-node` (primary)

## Files Changed
- `crates/whirlpool-node/src/main.rs` — Node wiring and dependency injection.

## Dependencies
- Task 01: InMemoryTxPool implementation + unit tests

## Design Refs
- `docs/design/evmblock-txsource/FLOWS.md S-3, F3`

## TDD Sequence
1. **Red**: Update `main.rs` to attempt using `InMemoryTxPool` before it is implemented/imported.
2. **Green**: Instantiate `InMemoryTxPool` in `main.rs`, wrap it in an `Arc`, and pass it to `EvmApplication::new()`.
3. **Verify**: Run AC commands to ensure the node builds successfully.

## Implementation Details
In the node's `main()` function, create an `Arc<InMemoryTxPool>`. Use this handle as the transaction source when initializing the `EvmApplication`. The application needs this handle to retrieve pending transactions when proposing a new block.

## Acceptance Criteria
```
nix develop --command cargo build -p whirlpool-node --bin whirlpool-node
grep -q 'InMemoryTxPool::new()' crates/whirlpool-node/src/main.rs
grep -q 'InMemoryTxPool' crates/whirlpool-node/src/main.rs | grep -v NoopTxSource
```

## Evidence
- `.sisyphus/evidence/evmblock-txsource/02-node-wiring.log`
