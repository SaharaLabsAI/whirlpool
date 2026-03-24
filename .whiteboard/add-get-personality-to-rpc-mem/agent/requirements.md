# Requirements

## Intent
Implement a read endpoint in `rpc-mem` to fetch the latest finalized personality by `personality_id`.

## Scope (REQ)
- REQ-1: Expose a new JSON-RPC method in `rpc-mem` for personality reads.
- REQ-2: Add a service-layer read contract so RPC handlers do not couple directly to storage internals.
- REQ-3: Return deterministic `None/not found` behavior when a personality is absent.
- REQ-4: Keep existing `mem_submitPersonality` behavior unchanged.
- REQ-5: Reuse the existing finalized personality model from `state::StoredPersonality`.
- REQ-6: Keep wire-format validation deterministic for request fields.
- REQ-7: Cover happy path and not-found path with rpc-mem tests.

## Assumptions
- Personality data source remains finalized storage semantics (`state::PersonalityStorage`), not mempool state.
- MVP lookup key is `personality_id` only.
- Read API can return markdown and metadata required for clients to identify freshness.

## Non-goals
- No query-by-signer/nonce endpoint in this change.
- No pagination or list endpoint.
- No signature verification redesign.

## Success Criteria
- A client can call the new method and receive latest finalized personality data when present.
- Missing personalities return a stable not-found/null contract.
- Existing submit flow tests remain green.
