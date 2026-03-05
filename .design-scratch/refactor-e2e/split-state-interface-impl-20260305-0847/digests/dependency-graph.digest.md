# dependency-graph.digest

- **Grounded**: `state` is a current workspace member with direct deps on `revm`, `sha2`, `thiserror`, and `alloy-genesis`; it currently exports both interface (`StateDb`, `StateError`) and concrete in-memory DB symbols from one crate.
- **Grounded**: `state-memory` is not present yet and is explicitly `[PROPOSED]/missing`; it is expected to host `DbAccount`, `InMemoryStateDb`, and `revm::Database`/`DatabaseRef` impls while depending on `state`.
- **Grounded**: Reverse dependency pressure is concentrated in `app-evm` and `whirlpool-node`, both of which currently import `state::InMemoryStateDb`; these imports are the primary migration seam.
- **Grounded**: `state` has no crate-local feature flags and workspace feature wiring is minimal (`resolver = "2"`), so the split is dominated by path/dependency updates rather than feature-matrix changes.
- **Grounded**: Circular dependency risk is well-defined: `state` must remain interface-only with one-way edge `state-memory -> state`; re-exporting concrete DB from `state` would reintroduce interface/implementation coupling.
- **Verification**: Circular dependency check ran successfully via `cargo metadata` graph traversal; result `CIRCULAR_DEPENDENCY_CHECK: PASS` (no workspace cycles detected).
- **[PROPOSED]**: Keep migration ordering as `state` interface stabilization first, then introduce `state-memory`, then update consumers (`app-evm`, `whirlpool-node`) to concrete imports.
- **BLOCKER**: None from dependency topology at explore digest stage.
