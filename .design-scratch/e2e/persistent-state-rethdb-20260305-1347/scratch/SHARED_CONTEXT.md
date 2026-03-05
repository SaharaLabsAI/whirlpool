# SHARED CONTEXT

## Workspace Overview (Intake Initialization)
- Workspace root: `/home/dev/sahara/web3/agent/playground/whirlpool`
- Intent topic: persistent state via reth-db (MDBX)
- Alignment iteration: `1`
- Depth: `module`
- Focus crates: `state`, `state-memory`, `whirlpool-node`

## Known Architecture Snapshot (Provided Facts)
- `state` defines `StateDb` with 11 methods (`new`, `with_genesis`, `state_root`, `commit`, read accessors, insert helpers).
- `state-memory` provides `InMemoryStateDb` (HashMap-backed), implementing `StateDb` + `revm::Database` + `revm::DatabaseRef`.
- `whirlpool-node` currently wires a `TestStateDb` wrapper around `InMemoryStateDb`, shared via `Arc<RwLock<_>>` into EVM app and RPC.
- `app-evm` and `rpc-eth` are generic over `S: StateDb` and should remain implementation-agnostic.
- Vendored reth storage ecosystem is available (`reth-db`, `reth-db-api`, `reth-provider`, `reth-storage-api`, `libmdbx-rs`, and trie crates).

## Intake Boundary Map (Module Depth)
- Primary new crate boundary: `state-reth` (MDBX-backed `StateDb` implementation).
- Modified composition boundary: `whirlpool-node` (swap state backend wiring away from in-memory test wrapper).
- Touched interface boundary: `state` (trait may need minimal fallibility-oriented adjustments if required by persistent backend integration).
- Behavioral reference boundary: `state-memory` (semantic parity target for baseline state operations).
- Downstream consumer boundaries: `app-evm`, `rpc-eth` (generic users of `StateDb`, no implementation-specific coupling expected).

## Breadth / Threshold Check
- crates_count: `6` (`state`, `state-memory`, `state-reth`, `whirlpool-node`, `app-evm`, `rpc-eth`) → exceeds threshold `>3`
- boundaries_count: `6` → exceeds threshold `>4`
- domains_count: `3+` (state interface, persistent storage backend, node/runtime wiring) → exceeds threshold `>2`
- flows_count: `4+` (DB init/open, genesis initialization, commit/read lifecycle, app+RPC shared state access) → exceeds threshold `>3`

Result: scope is flagged **too broad** and must stay tightly constrained to module-depth alignment.

## Intake Constraints (Enforced)
- No internal crate exploration in this phase.
- No strategy/domain/flow synthesis docs in this phase.
- No cargo commands in this phase.
