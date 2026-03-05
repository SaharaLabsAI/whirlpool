# impact-analysis.digest

- **Grounded**: The highest-impact symbol move is `InMemoryStateDb` (`crates/state/src/db.rs` -> future `state_memory::db`), because it is imported across `app-evm` execution/tests and `whirlpool-node` runtime wiring.
- **Grounded**: `StateDb` and `StateError` remain in `state`, with `DBErrorMarker` retained alongside `StateError`; this preserves trait/error contracts while relocating concrete storage implementation.
- **Grounded**: `DatabaseRef` and `Database` impl relocation is medium-to-high risk because `revm` trait conformance is sensitive to visibility and error type wiring.
- **Grounded**: Cross-crate seam changes are explicit: introduce `state-memory` with one-way dependency on `state`; update concrete consumers to new import paths; keep `state` free of reverse dependency on `state-memory`.
- **Grounded**: Architectural breakage classes are mostly compile-time detectable (unresolved imports, missing impls, trait-bound failures), concentrated in `app-evm` and `whirlpool-node`.
- **[PROPOSED]**: Preserve temporary compatibility only where needed for migration; avoid keeping long-lived concrete re-exports in `state` to satisfy interface-first objective.
- **UNKNOWN**: Extent of non-code references (docs/scripts) requiring path updates beyond known crate callsites.
- **BLOCKER**: None; risks are migration-order and completeness risks, not design impossibilities.
