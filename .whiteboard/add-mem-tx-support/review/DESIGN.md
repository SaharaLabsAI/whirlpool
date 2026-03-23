# Design Review

## Decision
PASS for finalize-phase approval, with the scope fixed to prototype mem/personality transaction support and the existing alignment baseline preserved.

## What Is Approved
- Add `crates/app-mem` for non-EVM mem transaction schema, structural validation, and finalized-write derivation.
- Add `crates/rpc-mem` for `mem_submitPersonality`, separate from `crates/rpc-eth`.
- Keep `app::traits::TxSource` and `crates/mempool-mdbx` payload-agnostic in v1.
- Extend mixed proposal/verification logic around `crates/app-evm/src/executor.rs` so mem payloads are classified deterministically and EVM execution semantics remain intact.
- Let `crates/whirlpool-node/src/node.rs` own dual RPC startup, the prototype in-memory personality store, and finalization-time write flushing near `crates/whirlpool-node/src/persisting_sink.rs`.

## Why This Passes
- It stays consistent with `.whiteboard/personality-markdown-tx/review/DESIGN.md` and the approved alignment digest.
- It uses the already-documented `SignedTransaction::Other { type_id, payload }` extension path from `docs/chain/crates/types.md`.
- It preserves Whirlpool's canonicality boundary by making personality state visible only after finalization.
- It contains experimental mem behavior in new crates instead of widening Ethereum-specific contracts.

## Required Guardrails
- Keep TST-001 through TST-007 unchanged as the baseline acceptance suite.
- Do not overstate signature verification; v1 is structural-only and Jolt remains deferred.
- Keep last-finalized-write-wins semantics per `personality_id` explicit.
- Keep restart volatility and in-memory capacity limits explicit in every downstream plan.

## Deferred By Design
- Durable personality storage.
- Retrieval/query RPC.
- Strong replay and pending replacement policy.
- Jolt-backed cryptographic authorization.

## Approval Note
This design package is decision-complete for planning. Any later change that merges mem RPC into `rpc-eth`, splits the mempool in v1, or exposes pre-finalization personality state should be treated as a design regression and re-reviewed.
