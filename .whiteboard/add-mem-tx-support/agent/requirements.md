# Requirements

## Scope
- Depth: `module`
- Focus crates: `all`
- Intake breadth check: **within threshold**
  - Crates affected: 2 new primary crates + 3 integration boundaries (`crates/app-mem`, `crates/rpc-mem`, `crates/app`, `crates/app-evm`, `crates/whirlpool-node`)
  - Boundaries: 4 (RPC ingress, shared mempool ingress, mixed transaction application path, finalization storage)
  - Domains: 2 (non-EVM transaction ingestion and finalization-side memory storage)
  - Flows: 3 (RPC submission, proposal/verification classification, finalization persistence)
- Intent split decision: **no split required**

## Requirements
- REQ-1: Add first-class mem/personality transaction support through a dedicated RPC surface instead of `rpc-eth`.
- REQ-2: Introduce `crates/rpc-mem` for experimental mem-facing JSON-RPC methods, including submission of personality markdown transactions.
- REQ-3: Introduce `crates/app-mem` for non-EVM mem transaction types, structural validation, and block-side finalized-write derivation.
- REQ-4: Keep the shared mempool ingress path generic via `TxSource`; mem transactions must reuse the opaque-byte queue.
- REQ-5: Proposal and verification must classify raw pending bytes deterministically into EVM transactions, mem transactions, or invalid payloads without regressing EVM execution behavior.
- REQ-6: Personality data must become visible only after block finalization through a dedicated prototype in-memory store with last-finalized-write-wins semantics per `personality_id`.
- REQ-7: `whirlpool-node` must own the mem/personality store and run both `rpc-eth` and `rpc-mem` server lifecycles against shared chain dependencies.
- REQ-8: v1 validation must include deterministic structural checks for mem transactions while explicitly deferring cryptographic/Jolt verification.
- REQ-9: Leave a clean future boundary for replay protection, durable storage, retrieval RPC, and Jolt-backed verification.

## Assumptions
- `.whiteboard/personality-markdown-tx/review/DESIGN.md` is the grounded starting point for the design.
- Existing `TxSource` opaque-byte semantics are stable enough to carry mixed EVM and mem transaction families.
- Prototype persistence may stay in-memory and node-local for v1 as long as finalization-only visibility is preserved.

## Non-Goals
- Personality execution semantics inside the EVM.
- Durable personality storage in v1.
- Mandatory Jolt proof generation or verification in v1.
- Replacing `rpc-eth` or changing Ethereum compatibility beyond shared node wiring.
- Broad retrieval/query APIs beyond what later phases explicitly scope.

## Success Criteria
- `rpc-mem` and `app-mem` responsibilities are clearly separated from Ethereum-oriented crates.
- Mixed-transaction proposal/verification rules are deterministic and preserve current EVM behavior.
- Finalized personality writes land only in the new prototype memory store after finalization.
- Node wiring for dual RPC servers and shared dependencies is fully covered before implementation planning.
