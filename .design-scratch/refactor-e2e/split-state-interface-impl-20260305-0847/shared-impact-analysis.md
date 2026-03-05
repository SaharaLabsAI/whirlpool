**Phase 02 Step 2 Impact Analysis**

Scope:
- Depth: Architectural (crate boundaries, dependency edges, public API surface, reverse dependencies).
- Crates: `state` (interface-only), upcoming `state-memory` (concrete `InMemoryStateDb`/revm wiring), plus consumers `app-evm` and `whirlpool-node` that pull the current concrete DB.
- Symbols: `StateDb`, `StateError`, `DBErrorMarker impl`, `DbAccount`, `InMemoryStateDb`, `DatabaseRef impl`, `Database impl`.

### Symbol impacts

1. **StateDb**
   - Current definition: `crates/state/src/traits.rs` defining the trait and `state::traits::StateDb` re-exported in `crates/state/src/lib.rs` while `crates/state/src/db.rs` implements it for `InMemoryStateDb`.
   - Target change: Remains in the `state` interface crate so consumers continue to depend on `state::traits::StateDb` without dragging in implementation details.
   - Consumer impact: `crates/app-evm/src/traits.rs` bounds `StateProvider` on `StateDb`, and `crates/app-evm/src/executor.rs` uses that trait to constrain `EvmApplication<DB>` (line 82). `state::InMemoryStateDb` currently satisfies the trait.
   - Risk: Medium (trait-bound change hits the `EvmApplication` generic, so any mistakes in re-exporting or trait visibility could break compile-time guarantees across the `app-evm` crate).

2. **StateError**
   - Current definition: `crates/state/src/error.rs` with `StateError` re-exported via `state::StateError` and consumed by the `revm` trait impls inside the same crate.
   - Target change: Stays in the `state` crate (interface surface) so downstream `state-memory` and runtime crates keep using `state::StateError` for error handling.
   - Consumer impact: `InMemoryStateDb`'s `Database`/`DatabaseRef` impls use `StateError` as `type Error` (lines 185, 212 of `crates/state/src/db.rs`), and `crates/whirlpool-node/src/main.rs` exposes `type Error = state::StateError` for its `TestStateDb` `revm::Database` implementation (lines 47-74).
   - Risk: Low-to-medium (only needs consistent re-export and the `StateError` type to remain public; losing it would break the runtime's `revm::Database` wiring).

3. **DBErrorMarker impl**
   - Current definition: `impl revm::database::DBErrorMarker for StateError` in `crates/state/src/error.rs` so `StateError` can serve as `revm::Database` error.
   - Target change: Must remain alongside `StateError` in the `state` crate (interface) despite the split.
   - Consumer impact: `revm::Database` and `DatabaseRef` impls for `InMemoryStateDb` (lines 185-230) depend on this implementation to satisfy `revm`'s marker trait; `TestStateDb` in `crates/whirlpool-node/src/main.rs` transitively relies on the same error compatibility when delegating to `InMemoryStateDb`.
   - Risk: Low (the implementation is trivial but critical for compatibility; forgetting to keep it public would break the revm trait impls in `state-memory`).

4. **DbAccount**
   - Current definition: `crates/state/src/db.rs::DbAccount` (struct used to store per-account `AccountInfo` + storage map) and exported through `state::db` module.
   - Target change: Move the struct into `state_memory::db` so all concrete DB internals live in the implementation crate along with `InMemoryStateDb`.
   - Consumer impact: `InMemoryStateDb` uses `DbAccount` for `accounts.insert`/`entry` paths (`with_genesis`, `commit`, `insert_account`), and unit tests defined in `state/src/db.rs` (lines 244 onwards) expect the type to still be accessible in the same module after the split.
   - Risk: Low (pure data type, but moving it requires updating `state` tests and any `state`-owned helpers that relied on the module path or `pub use`).

5. **InMemoryStateDb**
   - Current definition: `crates/state/src/db.rs::InMemoryStateDb` struct plus public helpers (new, with_genesis, commit, state_root) and re-export via `state::db`.
   - Target change: Relocate to `state_memory::db::InMemoryStateDb`, with only trait/interface references remaining in `state` (`StateDb`, `StateError`).
   - Consumer impact: `crates/app-evm/src/executor.rs` and its tests (e.g., `setup_app`) instantiate `Arc<RwLock<InMemoryStateDb>>` for `EvmApplication`, and `crates/whirlpool-node/src/main.rs` wraps it in `TestStateDb` before providing it to the runtime (lines 28-135). All these places will need to depend on `state-memory` instead of pulling `InMemoryStateDb` from `state`.
   - Risk: High (breaking change across multiple crates; forgetting to update dependency entries or use statements will fail compilation, and downstream docs/tests referencing `state::InMemoryStateDb` must be migrated to `state_memory::InMemoryStateDb`).

6. **DatabaseRef impl**
   - Current definition: `crates/state/src/db.rs::impl DatabaseRef for InMemoryStateDb` (lines 185-211) inside the `state` crate.
   - Target change: Move this implementation to `state_memory::db` alongside `InMemoryStateDb`, so the concrete crate provides the `revm` read-only interface.
   - Consumer impact: `revm::State` builder in `crates/app-evm/src/executor.rs` (lines 144-148 primarily) uses the `DatabaseRef` impl during execution; `whirlpool-node` uses `TestStateDb` (wrapping `InMemoryStateDb`) to satisfy `revm::Database` by delegating to those `DatabaseRef` methods.
   - Risk: Medium (the `revm` traits are sensitive to visibility and crate layout; the `state-memory` crate must continue to expose the same `revm` impls with `StateError` from the interface crate).)

7. **Database impl**
   - Current definition: `crates/state/src/db.rs::impl Database for InMemoryStateDb` (lines 213-231).
   - Target change: Move to `state_memory::db` so the implementation crate provides the mutable `revm::Database` interface.
   - Consumer impact: `TestStateDb` in `crates/whirlpool-node/src/main.rs` forwards its `revm::Database` calls to the concrete `InMemoryStateDb` implementation; `app-evm` indirectly relies on `revm::Database` to execute transactions inside `State::builder().with_database(&mut state_snapshot)` (line 145).
   - Risk: Medium-high (misplacing the `revm::Database` impl or forgetting to re-export it will break the runtime wiring for both `app-evm` and `whirlpool-node`).

### Cross-crate seam analysis

- **state → state-memory**: The split introduces a new dependency edge where `state-memory` depends on `state` for `StateDb`, `StateError`, `BundleState`, and other `revm` primitives. The interface crate must drop the concrete exports (`InMemoryStateDb`, `DbAccount`, `Database/DatabaseRef` impls) so interface-only consumers no longer pull implementation code. `state-memory` becomes the new home of the concrete DB plus `revm` trait impls and will re-export `state_memory::db::{DbAccount, InMemoryStateDb}` for convenience. Migration risk occurs wherever the repository currently imports `state::InMemoryStateDb`/`state::DbAccount` (e.g., `crates/app-evm`, `crates/whirlpool-node`, `ldocs`).

- **state → app-evm**: `app-evm` relies on `state::StateDb` for trait bounds via `app-evm/src/traits.rs` and now imports `state::InMemoryStateDb` when instantiating the canonical execution state (see tests in `crates/app-evm/src/executor.rs` and `tests/*`). After the split, these imports must pivot to `state_memory::InMemoryStateDb` while keeping `StateDb` on the interface crate. Additional risk arises from any doc strings or macros expecting `state::InMemoryStateDb` (these need updates).

- **state → whirlpool-node**: `whirlpool-node/src/main.rs` uses `state::InMemoryStateDb` inside `TestStateDb`, implements `StateProvider` for `TestStateDb` (which in turn becomes `StateDb` once `app-evm` trait relocation finishes), and exposes `state::StateError` via `revm::Database`. After the split, `whirlpool-node` must depend on `state-memory` for the actual DB type and keep depending on `state` solely for the interface traits and errors.

### Architectural breakage classes

- **Import path shifts**: References such as `use state::InMemoryStateDb` (e.g., `crates/app-evm/src/executor.rs`, `crates/app-evm/tests/*`, `crates/whirlpool-node/src/main.rs`) must change to `state_memory::InMemoryStateDb`. Compatibility re-exports may be introduced temporarily but must be removed to keep the interface crate lean.

- **API/trait bounds**: `StateDb` remains the trait exposed by `state`, so `EvmApplication<DB>` still requires `DB: StateProvider + ...` in `crates/app-evm/src/executor.rs`. Ensuring the trait stays public and reachable (no crate relocation) is critical because downstream bounds (e.g., `state::traits::StateDb` in `app-evm/src/traits.rs`) anchor the entire execution stack.

- **Crate dependency edges**: The interface crate (`state`) must not depend on `state-memory`, but `state-memory` depends on `state`. Consumers that once pointed at `state` for the concrete DB (app-evm tests, node binary) now need to add `state-memory` as a dependency to obtain `InMemoryStateDb` while still depending on `state` for the trait errors.

- **Runtime wiring**: `whirlpool-node`'s runtime wiring (lines 130-136 in `crates/whirlpool-node/src/main.rs`) instantiates `TestStateDb(InMemoryStateDb)` and feeds it to `EvmApplication`/`ApplicationAdapter`. If `TestStateDb` can no longer find `InMemoryStateDb` in `state`, the wiring must reference the new crate and keep the `revm::Database`/`StateProvider` layers intact.

### Unknowns / assumptions

- The exact layout of the new `state-memory` crate is not yet defined. We assume it will contain `state_memory::db::{DbAccount, InMemoryStateDb}`, re-exported via `state_memory::InMemoryStateDb` for parity with current usage, and will retain the `Database`/`DatabaseRef` impls and helper constructors (new, with_genesis, commit, state_root, insert_*).
- We assume no other crates besides `app-evm` and `whirlpool-node` import `state::InMemoryStateDb`/`DbAccount` directly; if there are additional consumers (e.g., docs, tests, scripts), they must be reviewed for import-path updates.
- The interface crate will continue to depend on `revm` for trait signatures (`BundleState`, `AccountInfo`, `Bytecode`), so splitting the implementation should not change `state`'s existing dependencies, but we must ensure `state-memory` also brings in the same `revm` types without duplicating functionality.
- We assume the `state-memory` crate will expose the same `pub fn new()/with_genesis()` helpers so downstream consumers can use the familiar constructors without renaming.
