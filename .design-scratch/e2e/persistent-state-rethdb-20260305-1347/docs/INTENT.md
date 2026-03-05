# INTENT

## Parsed Objective
Add persistent node state storage backed by `reth-db` (MDBX) by introducing a new `state-reth` crate that implements the existing `StateDb` trait, then wire `whirlpool-node` to use this persistent backend instead of the in-memory `TestStateDb` wrapper.

## Concrete Requirements
- Create a new crate: `state-reth`.
- Implement `state::StateDb` in `state-reth` using MDBX persistence via vendored reth storage stack.
- Keep `state` as the interface crate; if MDBX/reth integration requires error propagation, allow minimal trait-surface adjustments (fallibility-focused, tightly scoped).
- Use `state-memory` as the behavioral baseline for StateDb semantics.
- Modify `whirlpool-node` wiring so runtime state is backed by the new persistent implementation (replace `TestStateDb` + `InMemoryStateDb` usage path).
- Maintain compatibility for `app-evm` and `rpc-eth`, which consume `StateDb` generically.
- Respect depth scope: `module` (no full-system redesign during this phase).

## Affected Crate Boundaries (Initial)
- `state` (interface boundary): trait authority for `StateDb`; expected unchanged.
- `state-memory` (reference boundary): existing HashMap implementation used as semantics baseline.
- `state-reth` (new implementation boundary): MDBX-backed `StateDb` implementation and storage adapter layer.
- `whirlpool-node` (composition/wiring boundary): chooses concrete state backend and dependency injection path.
- `app-evm` (consumer boundary): generic `StateDb` user; should remain implementation-agnostic.
- `rpc-eth` (consumer boundary): generic `StateDb` user; should remain implementation-agnostic.

## Threshold Check (Breadth Gate)
Given the known scope:
- crates: **6** (`state`, `state-memory`, `state-reth`, `whirlpool-node`, `app-evm`, `rpc-eth`) → exceeds `>3`
- boundaries: **6** → exceeds `>4`
- domains: **3+** (state interface, storage backend, node wiring/runtime composition) → exceeds `>2`
- flows: **4+** (init/open DB, genesis bootstrapping, commit/read path, app+RPC access path) → exceeds `>3`

**Result:** scope is **too broad** for a single unconstrained pass; alignment should keep implementation focused at module depth with strict boundary control.

## In-Scope / Out-of-Scope for Intake
- In-scope now: intent parsing, boundary identification, breadth flagging.
- Out-of-scope now: strategy synthesis, domain modeling, flow synthesis, internal crate exploration.
