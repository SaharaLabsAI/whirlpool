# module-structure.digest

- **Grounded**: `app` mixes interfaces and concrete tx-source implementations in `src/traits.rs`; `src/adapter.rs` is implementation glue into consensus.
- **Grounded**: `consensus` trait definitions are already isolated by concern files (`app.rs`, `block.rs`, `event.rs`, `engine.rs`) but not yet consolidated into a dedicated trait namespace module.
- **Grounded**: `p2p` already has a clean trait/data split (`traits.rs`, `types.rs`) with concrete adapters externalized to `p2p-commonware`.
- **Grounded**: `state` currently exports concrete DB types from `db.rs` without a local trait abstraction module.
- **Grounded**: `consensus-simplex` defines interface-like `CommonwareBlock` in `types.rs` while operational logic remains in `adapter.rs`, `engine.rs`, `mailbox.rs`, `sink.rs`.
- **Grounded**: `app-evm` defines `StateProvider` in `executor.rs` beside concrete `EvmApplication` implementation, making it a prime split target.
- **[PROPOSED]**: Standardize all crates to explicit interface module + implementation modules with temporary compatibility re-exports.
- **UNKNOWN**: Final crate-internal module naming conventions (`traits`, `interface`, or split-by-domain) to be selected in synthesis.
- **BLOCKER**: None for explore.
