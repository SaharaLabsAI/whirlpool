# Alignment Digest

## Approved Intent
Add mem/personality transaction support to Whirlpool using the prior design in `.whiteboard/personality-markdown-tx/review/DESIGN.md`, while explicitly introducing a new `app-mem` crate to separate non-EVM transaction logic from `app-evm`.

## Confirmed Scope
- **New crates**: `crates/app-mem`, `crates/rpc-mem`
- **Primary shared boundaries**: `crates/app`, `crates/app-evm`, `crates/whirlpool-node`
- **Likely storage touch**: dedicated prototype personality store, potentially under `crates/state` or `crates/state-memory`
- **Shared mempool**: keep `TxSource` and `mempool-mdbx` payload-agnostic in v1
- **No split required**: module-scale feature with bounded cross-crate integration

## Approach Direction
1. Keep `rpc-eth` Ethereum-only and add a separate `rpc-mem` submission surface.
2. Add `app-mem` for mem transaction payloads, structural validation, and finalized-write derivation.
3. Preserve the generic opaque-byte mempool ingress and classify pending bytes deterministically in proposal/verification.
4. Wire a prototype in-memory personality store through `whirlpool-node` and flush writes only on finalization.
5. Preserve explicit future boundaries for replay hardening, durable storage, read RPC, and Jolt-backed verification.

## Risks
- **R1 (medium)**: mixed transaction classification touches EVM-centric application code and can regress existing execution semantics if separation is weak.
- **R2 (medium)**: prototype personality storage is in-memory only, so restart loss and growth limits must remain explicit.
- **R3 (medium)**: v1 signature handling is structural-only; authenticity is deferred and must not be overstated.
- **R4 (low)**: replay/dedup rules may remain intentionally minimal in v1 and require later hardening.

## Iteration Count
1
