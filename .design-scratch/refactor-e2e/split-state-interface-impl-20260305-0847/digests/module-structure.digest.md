# module-structure.digest

- **Grounded**: `crates/state/src/lib.rs` currently exposes `db`, `error`, and `traits`, then re-exports `DbAccount`, `InMemoryStateDb`, `StateError`, and `StateDb`; interface and concrete implementation are intentionally co-located today.
- **Grounded**: `state::traits` already contains the interface surface (`StateDb`) and is consumed downstream (notably by `app-evm::traits::StateProvider`), making it the natural anchor for an interface-only crate.
- **Grounded**: `state::db` owns all in-memory concrete behavior (`DbAccount`, `InMemoryStateDb`, `impl StateDb`, `impl DatabaseRef`, `impl Database`), so this module is the primary extraction target into the new implementation crate.
- **Grounded**: `state::error` defines `StateError` plus `DBErrorMarker`; the split intent keeps this in `state` to preserve the shared error contract.
- **Grounded**: Consumer module structure confirms runtime wiring dependence on concrete DB: `whirlpool-node/src/main.rs` wraps `InMemoryStateDb`, while `app-evm` tests instantiate it directly.
- **[PROPOSED]**: After split, `state` should stop re-exporting concrete symbols and export interface/error only; `state-memory` should provide concrete re-exports to minimize downstream churn.
- **UNKNOWN**: Final public path naming convention (`state_memory::...` vs alternative facade naming) is not yet fixed by synthesis.
- **BLOCKER**: None in module decomposition.
