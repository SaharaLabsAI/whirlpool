# WORKSPACE

## Updated Workspace Members

Add the new crate to workspace membership:

```toml
[workspace]
members = [
    "crates/consensus",
    "crates/consensus-simplex",
    "crates/p2p",
    "crates/p2p-commonware",
    "crates/rpc-eth",
    "crates/whirlpool-node",
    "crates/state",
    "crates/state-memory",
    "crates/state-reth",
    "crates/app",
    "crates/app-evm",
]
```

No other existing members are removed by this design step.

---

## Integration Topology

### High-level dependency graph

```text
state (trait contract)
  |\
  | \-> state-memory (reference impl; kept for tests/dev)
  |
  \--> state-reth (persistent MDBX impl)
         |
         +--> reth-db + reth-db-api + reth-storage-errors + reth-codecs + reth-trie

app-evm ----(generic over StateDb/revm::Database)---->
rpc-eth ----(generic over StateDb)-------------------->

whirlpool-node
  |- depends on app/app-evm/rpc-eth/state
  |- chooses concrete backend: state-reth (runtime default target)
  \- shares backend via Arc<RwLock<...>> across EVM + RPC
```

### Crate edge table

| From crate | To crate | Edge type | Notes |
|---|---|---|---|
| `state-reth` | `state` | workspace | Implements trait contract |
| `state-reth` | `reth-db` stack | vendored external | MDBX/table/trie integration |
| `whirlpool-node` | `state-reth` | workspace | Concrete persistent backend wiring |
| `whirlpool-node` | `state-memory` | workspace | Optional retention for tests/fallback only |
| `app-evm` | `state` | workspace | Generic trait boundary remains |
| `rpc-eth` | `state` | workspace | Generic trait boundary remains |

---

## Build Order Considerations

Cargo will compute full DAG order automatically; preferred conceptual order for integration rollout:

1. `state` (finalize `StateDb` signature contract)
2. `state-memory` (adapt to fallible trait with infallible error type)
3. `state-reth` (new persistent implementation)
4. `app-evm` + `rpc-eth` (compile and adapt error propagation where required)
5. `whirlpool-node` (switch runtime wiring to `state-reth`)

Ordering rationale:

- Trait contract must stabilize first because all backend and consumer crates compile against it.
- `state-reth` depends on resolved fallibility contract (BLK-001).
- Node wiring should happen after both backend and consumer compile paths are aligned.

---

## Feature Flag Plan

### `state-reth` crate features

Proposed features:

```toml
[features]
default = ["mdbx"]
mdbx = ["reth-db/mdbx"]
```

Guidance:

- Keep `mdbx` as default for this initiative because persistent backend target is MDBX.
- Keep feature surface minimal; avoid pulling broader `reth-provider` stack unless needed.
- Explicitly document that disabling default features is unsupported for primary node runtime path unless an alternate backend feature is introduced.

### Workspace-level feature posture

- Workspace currently has no centralized `[workspace.dependencies]`; keep crate-local feature control for this design step.
- If future unification is desired, centralize `revm`/`alloy-primitives`/reth storage dependencies after initial integration stabilizes.

---

## Workspace Cargo.toml Changes Needed

At workspace root (`/home/dev/sahara/web3/agent/playground/whirlpool/Cargo.toml`):

- Add `"crates/state-reth"` to `[workspace].members`.
- Keep `exclude = ["vendor"]` as-is.
- No mandatory `[patch]` additions required in this planning phase.
- No mandatory `[workspace.dependencies]` migration required in this planning phase.

Per-crate Cargo updates required for integration:

- `crates/state-reth/Cargo.toml`: add `state` + reth storage/trie/codec + `revm` + `thiserror` deps.
- `crates/whirlpool-node/Cargo.toml`: add `state-reth`; optionally retain or later remove `state-memory` depending on test/fallback posture.
- `crates/state/Cargo.toml`: likely unchanged unless trait/error bounds require additional imports only.

---

## Blocker-Driven Integration Constraints

Hard blockers from `BLOCKERS.md` affecting workspace integration:

- **BLK-001:** cannot finalize cross-crate compile contract until canonical fallible `StateDb` signature is approved.
- **BLK-002:** cannot finalize acceptance criteria for trie-root correctness until hashed-state input contract is pinned.
- **BLK-003:** cannot finalize runtime enablement policy until MDBX host prerequisites contract is specified.

Soft blockers that do not prevent workspace wiring draft:

- **BLK-101:** block-hash table mapping remains implementation-time choice.
- **BLK-102:** final error variant taxonomy can be refined during implementation.
- **BLK-103:** performance tuning (batching/caching) is deferred and should not alter external contracts.

---

## Integration Outcome Target

When this plan is implemented:

- Workspace contains a dedicated persistent backend crate (`state-reth`).
- Node runtime wiring in `whirlpool-node` uses `state-reth` instead of `TestStateDb`/in-memory path.
- `app-evm` and `rpc-eth` continue operating against generic `StateDb` boundary.
- Feature posture is explicit (`mdbx` default in `state-reth`) and consistent with vendored reth dependency model.
