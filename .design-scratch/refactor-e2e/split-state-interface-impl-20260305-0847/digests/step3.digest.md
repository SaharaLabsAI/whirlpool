# Step 3 Digest

- **Grounded**
  - `state` currently mixes interface and implementation concerns; interface symbols are `StateDb` and `StateError` with `DBErrorMarker` in `crates/state/src/traits.rs` and `crates/state/src/error.rs`.
  - Concrete symbols (`DbAccount`, `InMemoryStateDb`, `DatabaseRef`, `Database`) are currently in `crates/state/src/db.rs` and are directly consumed by `app-evm` and `whirlpool-node`.
  - Dependency-shape target remains one-way: `state-memory -> state`; reverse edge is forbidden.

- **[PROPOSED]**
  - `IMPACT.md` defines blast radius around concrete import/path migration while retaining stable interface paths in `state`.
  - `STRATEGY.md` adopts interface-first ordering: stabilize interface -> introduce `state-memory` concrete exports -> rewire consumers -> cleanup.
  - Risk controls prioritize compile gates at implementation move and consumer rewiring seams.

- **UNKNOWN**
  - Complete non-code reference inventory (`docs/scripts/examples`) for `state::InMemoryStateDb` paths.
  - Whether temporary compatibility re-exports are required versus direct atomic consumer switch.

- **BLOCKER**
  - None.
