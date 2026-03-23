# Requirements

## Scope
- Depth: `module`
- Focus crates: `all`
- Intake breadth check: **within threshold**
  - Crates affected: 2 new primary crates + 3 integration boundaries (`crates/app-mem`, `crates/rpc-mem`, `crates/app`, `crates/app-evm`, `crates/whirlpool-node`)
  - Boundaries: 4 (RPC ingress, shared mempool ingress, mixed transaction application path, finalization storage)
  - Domains: 2 (non-EVM transaction ingestion and finalization-side memory storage)
  - Flows: 3 (RPC submission, proposal/verification classification, finalization persistence)
- Intent split decision: **no split required** (single module-scale feature spanning a bounded set of crates)

## Affected Boundaries
- New crate: `crates/app-mem` for non-EVM personality transaction types, validation, and finalized-write derivation.
- New crate: `crates/rpc-mem` for experimental memory/personality JSON-RPC methods.
- Existing shared boundary: `crates/app` for generic transaction-source abstractions and shared block/application types if mixed transactions require them.
- Existing EVM integration boundary: `crates/app-evm` because current proposal/verify logic assumes every pending transaction decodes as Ethereum.
- Existing node wiring boundary: `crates/whirlpool-node` for dual RPC server startup, store ownership, and finalization sink wiring.
- Existing persistence or prototype storage boundary: `crates/state` and/or `crates/state-memory` if a dedicated personality storage trait/backend is introduced there.
- Existing mempool boundary: `crates/mempool-mdbx` remains opaque-byte storage and should stay payload-agnostic.

## Requirements
- REQ-1: The workspace must add first-class mem/personality transaction support that can enter through a dedicated RPC surface instead of `rpc-eth`.
- REQ-2: The workspace must introduce `crates/rpc-mem` to own experimental mem-facing JSON-RPC methods, including initial submission of personality markdown transactions.
- REQ-3: The workspace must introduce `crates/app-mem` to hold non-EVM mem transaction data structures, structural validation, and block-side derived write logic so the logic is separated from `app-evm`.
- REQ-4: The shared mempool ingress path must remain generic opaque bytes via `TxSource`; mem transactions must not require a dedicated mempool implementation in v1.
- REQ-5: Proposal and verification logic must classify raw pending bytes deterministically into EVM transactions, mem transactions, or invalid payloads without regressing existing EVM execution behavior.
- REQ-6: Personality data must become visible only after block finalization through a dedicated prototype in-memory store with last-finalized-write-wins semantics per `personality_id`.
- REQ-7: `whirlpool-node` must own the new mem/personality store and run both `rpc-eth` and `rpc-mem` server lifecycles against shared chain dependencies.
- REQ-8: v1 validation must include deterministic structural checks for mem transactions (supported version, UTF-8 markdown, size limit, hash integrity, signature field presence/encoding) while explicitly deferring cryptographic/Jolt verification.
- REQ-9: The design must leave a clean future boundary for replay protection, durable storage, retrieval RPC, and Jolt-backed verification without blocking phase-1 implementation.

## Assumptions
- The prior design in `.whiteboard/personality-markdown-tx/review/DESIGN.md` is the primary alignment reference and is acceptable as a grounded starting point.
- Existing `TxSource` opaque-byte semantics are stable enough to carry mixed EVM and mem transaction families.
- Prototype persistence may stay in-memory and node-local for v1 as long as finalization-only visibility is preserved.

## Non-Goals
- Implementing personality execution semantics inside the EVM.
- Adding durable storage for personality data in v1.
- Adding mandatory Jolt proof generation or verification in v1.
- Replacing `rpc-eth` or altering Ethereum RPC compatibility beyond shared node wiring.
- Designing broad retrieval/query APIs beyond what the design phase explicitly scopes.

## Success Criteria
- The aligned design clearly separates `rpc-mem` and `app-mem` responsibilities from existing Ethereum-oriented crates.
- Mixed-transaction proposal/verification rules are deterministic and preserve existing EVM behavior.
- Finalized personality writes land only in the new prototype memory store after finalization.
- Node wiring for dual RPC servers and shared dependencies is fully covered before implementation planning.
