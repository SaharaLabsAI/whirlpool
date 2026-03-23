# Crate Contract: app-mem

## Purpose
Own non-EVM mem/personality transaction definitions, canonical encoding rules, structural validation, and finalized-write derivation so those concerns do not leak into `crates/app-evm`.

## Public Contract
- Define the mem transaction family carried under `SignedTransaction::Other { type_id, payload }`.
- Expose deterministic decode/validate helpers for proposal and verification.
- Derive finalized personality-write records from accepted mem transactions.
- Keep cryptographic proof verification out of v1; only structural signature checks are consensus-critical now.

## Invariants
- Exact submitted markdown bytes are the persisted bytes.
- Validation must be deterministic across proposal and verification.
- `markdown_hash` must bind the exact markdown bytes.
- Oversize, malformed, and unsupported-version payloads are rejected identically on every node.

## Out of Scope
- EVM execution semantics.
- RPC transport and server lifecycle.
- Durable storage backend.
- Jolt proof verification beyond future-facing field and message-binding boundaries.

## Required Integrations
- `crates/app`: shared transaction-source boundary stays opaque-byte based.
- `crates/app-evm`: mixed block path calls into `app-mem` for mem classification and validation.
- `crates/whirlpool-node`: consumes derived mem writes during finalization.
