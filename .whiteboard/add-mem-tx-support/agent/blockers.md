# Blockers

## Gate
PASS

## Hard Blockers
- None. The current workspace already has the necessary extension seams: generic raw-byte mempool ingress in `crates/app/src/traits.rs`, centralized node wiring in `crates/whirlpool-node/src/node.rs`, and a finalization persistence hook in `crates/whirlpool-node/src/persisting_sink.rs`.

## Active Risks To Manage
- `crates/app-evm/src/executor.rs` currently assumes all block transactions are EVM-decodable during `verify()`, so mixed transaction support must change that behavior carefully to avoid EVM regressions.
- Prototype personality storage is in-memory only, so restart loss and unbounded growth remain accepted v1 risks.
- Signature handling is structural-only in v1, so documentation and RPC behavior must avoid overstating authenticity guarantees.
- Replay and dedup policy can remain minimal initially, but the boundary for later hardening should be designed in now.

## Exit Criteria For Design
- New crate ownership is explicit: `app-mem` for non-EVM transaction logic and `rpc-mem` for experimental submission RPC.
- Mixed-transaction proposal/verification rules are deterministic and preserve existing EVM execution semantics.
- Finalization-only personality persistence path is clearly owned by node wiring and separate from `BlockStorage`.
- Workspace membership and cross-crate integration points are identified before implementation starts.
