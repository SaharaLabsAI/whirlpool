# Crate Contract: whirlpool-node

## Purpose
Own the composition boundary for dual RPC servers, shared mempool ingress, application wiring, and finalization-time personality persistence.

## Public Contract
- Instantiate shared chain dependencies already centered in `crates/whirlpool-node/src/node.rs`.
- Start both `rpc-eth` and `rpc-mem` against the same chain process.
- Own the prototype in-memory personality store and pass it to finalization handling.
- Preserve current finalized-block persistence while adding finalized mem-write flushing.

## Invariants
- `rpc-eth` stays Ethereum-only.
- Personality data is not externally visible before finalization.
- Shared `TxSource` remains generic opaque bytes.
- Node wiring, not leaf crates, owns lifecycle and cross-crate dependency assembly.

## Out of Scope
- Defining mem transaction canonical rules.
- Replacing `PersistingFinalizationSink` behavior for existing EVM block storage.
- Durable personality storage in v1.

## Required Integrations
- `crates/app-evm` plus `crates/app-mem` for mixed proposal/verification behavior.
- `crates/rpc-eth` and `crates/rpc-mem` for dual server startup.
- Prototype personality storage adjacent to existing `state::BlockStorage` ownership.
