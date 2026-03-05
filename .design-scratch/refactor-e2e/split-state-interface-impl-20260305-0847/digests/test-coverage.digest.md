# test-coverage.digest

- **Grounded**: Current validation coverage is strongest in `state/src/db.rs` unit tests and `app-evm` executor tests, which jointly exercise `StateDb`, `InMemoryStateDb`, state roots, commits, and `revm` read/write paths.
- **Grounded**: `StateError` and `DBErrorMarker` are primarily protected by compile-time conformance through `revm::Database`/`DatabaseRef` impl signatures and downstream generic bounds.
- **Grounded**: `whirlpool-node` has no dedicated test suite for `TestStateDb`; its safety currently depends on transitive compile/test coverage from shared crates.
- **Grounded**: Existing matrix identifies missing error-path coverage (especially `StateError::Internal`) and recommends explicit bridge tests for node-side delegation.
- **[PROPOSED]**: When `state-memory` is introduced, duplicate or relocate concrete DB behavior tests into that crate so trait contract parity remains explicit after extraction.
- **[PROPOSED]**: Add a compile-only contract test that imports `state::traits::StateDb` without concrete DB dependency to protect interface-only consumer goals.
- **UNKNOWN**: Whether all integration/doc examples referencing `state::InMemoryStateDb` are captured in current inventory.
- **BLOCKER**: None at explore digest phase; primary concern is post-move regression detection breadth.
