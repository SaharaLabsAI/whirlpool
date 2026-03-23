# Crate Contract: rpc-mem

## Purpose
Provide the experimental JSON-RPC surface for mem/personality transactions without extending Ethereum RPC semantics in `crates/rpc-eth`.

## Public Contract
- Expose `mem_submitPersonality` as the initial external method.
- Validate request shape, markdown size, UTF-8, supported version, and signature-field structure before enqueue.
- Canonically encode the mem payload and return a deterministic tx hash on acceptance.
- Reach the shared mempool only through a contained memory-ingress service adapter.

## Invariants
- `rpc-mem` never pretends a mem transaction is an Ethereum typed transaction.
- Admission checks must align with `app-mem` validation, not exceed or contradict it in consensus-visible ways.
- Failed admission produces no shared-mempool mutation.
- The crate remains submit-focused in v1; read APIs are deferred.

## Out of Scope
- Ethereum compatibility methods.
- Final block verification.
- Personality persistence.
- Direct mempool ownership or replacement policy.

## Required Integrations
- `crates/whirlpool-node`: starts the mem RPC server and injects the submission adapter.
- `crates/app`: adapter targets `TxSource` shared ingress.
- `crates/app-mem`: shared request/payload validation rules.
