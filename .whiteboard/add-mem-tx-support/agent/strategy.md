# Strategy

## Objective
Add first-class mem/personality transaction support without overloading the Ethereum path: keep `rpc-eth` Ethereum-only, add a dedicated `rpc-mem` ingress, preserve the shared opaque-byte mempool, classify mixed transactions deterministically during proposal/verification, and persist personality data only after finalization.

## Design Direction
- Keep `TxSource` generic over raw `Vec<u8>` payloads as already defined in `crates/app/src/traits.rs`; both the in-memory pool and MDBX pool already treat entries as opaque bytes and drain in FIFO order.
- Introduce `crates/app-mem` for personality transaction types, payload codec, structural validation, and finalized-write derivation so non-EVM logic does not accumulate inside `crates/app-evm`.
- Extend the application path so proposal and verification classify each raw payload into EVM, mem/personality, or invalid; preserve existing EVM execution for EVM transactions and use deterministic structural validation for mem transactions.
- Keep finalized writes out of proposal-time state changes; write personality content only from the finalization sink path, matching the current finalized block persistence flow.
- Add `crates/rpc-mem` as an experimental RPC surface for `mem_submitPersonality`, with `whirlpool-node` owning both RPC server lifecycles against the same chain, state, and mempool dependencies.

## Why This Fits The Current Workspace
- `crates/rpc-eth/src/server.rs` is tightly reth/Ethereum-oriented and bootstraps standard Ethereum modules, so non-EVM submission should not be forced through that surface.
- `crates/app-evm/src/executor.rs` currently assumes block transactions are EVM-decodable during `verify()` and filters undecodable mempool items during `propose()`, which is the main behavior that must be generalized.
- `crates/whirlpool-node/src/node.rs` already centralizes wiring for mempool, app, consensus, finalization, and RPC startup, making it the right place to own the new mem store and second RPC server.
- `crates/whirlpool-node/src/persisting_sink.rs` already persists finalized artifacts before delegating to the inner sink, which is the right extension point for finalization-only personality visibility.

## v1 Boundaries
- Structural validation only for personality transactions in consensus-critical paths.
- No personality execution semantics inside the EVM.
- No durable personality storage in v1.
- No retrieval/query RPC required for the first milestone.
- Leave replay hardening and Jolt-backed signature verification as explicit later phases.
