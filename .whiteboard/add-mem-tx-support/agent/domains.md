# Domains

## RPC Ingress
- Ethereum ingress stays in `crates/rpc-eth`, which is currently reth-backed and built for standard Ethereum module exposure.
- Mem/personality ingress moves to `crates/rpc-mem`, where request validation, canonical encoding, tx hash generation, and mempool submission can evolve without affecting Ethereum compatibility.

## Transaction Classification
- `crates/app/src/traits.rs` already gives a generic raw-byte boundary through `TxSource`.
- `crates/app-evm/src/executor.rs` is currently EVM-centric: `propose()` drains pending bytes and keeps only decodable EVM transactions, while `verify()` fails if block transactions are not EVM-decodable.
- The new classification domain is therefore a shared application concern: decode raw bytes into EVM, personality, or invalid and apply deterministic rules per family.

## Finalization Storage
- Finalized block persistence already happens in `crates/whirlpool-node/src/persisting_sink.rs` through `EvmApplication::store_finalized_block(...)` and `state::BlockStorage`.
- Personality persistence should follow the same finalization-only timing but land in a separate logical store, because `BlockStorage` is explicitly scoped to blocks and receipts.

## Prototype Personality State
- The prototype store should use in-memory semantics with last-finalized-write-wins per `personality_id`.
- Optional secondary indexing by `(signer, nonce)` can support later replay checks without changing the visible v1 model.
- Restart volatility is acceptable in v1 as long as it is explicit and not confused with durable chain state.

## Deferred Security Domain
- v1 consensus checks should cover payload decoding, supported version, UTF-8 validity, byte limits, and markdown-hash integrity.
- Cryptographic signer authorization and Jolt proof verification remain outside v1 and must be described as deferred, not implied.
