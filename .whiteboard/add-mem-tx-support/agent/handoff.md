# Design Handoff

## Build Order
1. Add `crates/app-mem` for mem payload types, codec, structural validation, and finalized-write derivation.
2. Add `crates/rpc-mem` with `mem_submitPersonality` and a memory-ingress service adapter over `TxSource`.
3. Relax the EVM-only assumption in proposal/verification so mixed raw bytes classify as EVM, mem, or invalid without changing existing EVM execution semantics.
4. Add the prototype in-memory personality store plus finalization-time flushing in `crates/whirlpool-node`.
5. Start dual RPC servers from `crates/whirlpool-node/src/node.rs` and keep `rpc-eth` unchanged in responsibility.

## Critical Decisions To Preserve
- Use the prior design in `.whiteboard/personality-markdown-tx/review/DESIGN.md` as the semantic baseline.
- Preserve the alignment QA baseline from `agent/tests.md` and `agent/testid-registry.md`.
- Keep mempool ingress payload-agnostic; do not split queues in v1.
- Persist personality content only after consensus finalization.
- Keep v1 signature checks structural-only and document that authenticity is deferred.

## Primary Workspace Touchpoints
- `crates/app/src/traits.rs`
- `crates/app-evm/src/executor.rs`
- `crates/rpc-eth/src/pool.rs`
- `crates/whirlpool-node/src/node.rs`
- `crates/whirlpool-node/src/persisting_sink.rs`

## Exit Condition
Design gate is satisfied when implementation planning can proceed without reopening crate ownership, canonicality boundaries, or the protected TST-001 through TST-007 baseline.
