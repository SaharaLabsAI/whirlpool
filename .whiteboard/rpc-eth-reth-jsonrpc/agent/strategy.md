# Strategy

## Objective
Produce a synth-only design for migrating `crates/rpc-eth` from the current hand-rolled JSON-RPC handlers to reth-backed `eth_*` RPC wiring, without implementation edits in `crates/*` during this phase.

## Scope and Limits
- In scope: design artifacts in `.whiteboard/rpc-eth-reth-jsonrpc/agent/`
- Out of scope: runtime implementation changes, `vendor/**` edits, non-`eth` namespaces
- Mandatory exclusion: blob/EIP-4844 support is explicitly unsupported in this integration

## Integration Strategy
Use reth's RPC stack as the method owner and keep Whirlpool logic in adapters:

1. `WhirlpoolProvider`
   - Bridges Whirlpool state/block backends into reth provider traits.
2. `WhirlpoolTxPool`
   - Bridges `app::traits::TxSource` into reth transaction-pool contract.
3. `WhirlpoolNetwork`
   - Satisfies required network-facing traits for module wiring.

Server composition is planned through `reth_rpc_builder::RpcModuleBuilder`, following reth's own test wiring pattern (`bootstrap_eth_api` + `build` + `RpcServerConfig::http(...).start(...)`).

## Builder Contract (Grounded)
`RpcModuleBuilder` imposes these key bounds for build-time composition:
- Provider: `FullRpcProvider + CanonStateSubscriptions + PersistedBlockSubscriptions + AccountReader + ChangeSetReader`
- Pool: `TransactionPool + Clone + 'static`
- Network: `NetworkInfo + Peers + Clone + 'static`

Design docs in this lane must keep adapter contracts aligned to those bounds.

## Adapter Behavior Contract

### `WhirlpoolProvider`
- Real/data-backed behavior for standard eth read paths (chain id, headers/blocks, transactions, receipts, account/state reads, block number/hash lookups).
- Stub/noop behavior only for surfaces not required by current Whirlpool RPC scope.
- Stub behavior must be deterministic and non-panicking (empty/none or explicit unsupported where appropriate).

### `WhirlpoolTxPool`
- Accepts raw tx input from RPC (`eth_sendRawTransaction`) and forwards to `TxSource`.
- Exposes pending transaction view from `TxSource::pending()`.
- Rejects unsupported transaction kinds at adapter boundary (see blob contract).
- Uses noop/minimal semantics for non-critical maintenance APIs where Whirlpool has no equivalent mempool operation.

### `WhirlpoolNetwork`
- Implements both `NetworkInfo` and `Peers` (not `NetworkInfo` alone).
- Returns static/minimal values analogous to reth noop behavior for RPC needs.
- No P2P ownership is introduced in `rpc-eth`; this remains an adapter for RPC trait satisfaction.

## Blob Exclusion Contract (Normative)
Blob support is intentionally excluded.

- `eth_blobBaseFee` exists in reth `eth` API surface, but Whirlpool contract is explicit: unsupported.
- Required behavior:
  - RPC returns an explicit unsupported-method/unsupported-feature style error contract.
  - No EIP-4844 execution/data path is added.
  - Type-3 blob transactions are rejected at `WhirlpoolTxPool` submission boundary.

This contract is required to be consistent across `requirements.md`, `tests.md`, and `blockers.md`.

## Migration Boundary
- `crates/rpc-eth`: internal module redesign from legacy context/handler/server wiring to adapter + builder composition.
- `crates/whirlpool-node`: RPC startup injection boundary update only (design in this phase).
- No other crate behavior redesign is included in this synth pass.

## Synthesize Completion Conditions
- Required synth artifacts exist and agree: `strategy.md`, `crates.md`, `workspace.md`, `domains.md`, `blockers.md`, `requirements.md`, `tests.md`, `run-state.md`.
- REQ-1..REQ-7 and TST-1..TST-12 are defined and mutually traceable.
- Blob exclusion contract is explicit, testable, and non-ambiguous.
