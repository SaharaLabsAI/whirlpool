# Strategy

## Objective
Add a read RPC method in `rpc-mem` that returns the latest finalized personality document for a provided `personality_id`.

## Implementation Direction
- Extend the rpc-mem service contract with a read operation that accepts decoded `personality_id` bytes.
- Keep the write ingress (`mem_submitPersonality`) unchanged.
- Bind read results to finalized storage semantics from `state::PersonalityStorage::get_latest`.
- Keep deterministic request validation and stable response encoding for binary fields.

## Scope Constraints
- No pending/mempool reads.
- No list/pagination surface.
- No additional personality mutation methods.

## Risks to Track
- Wire-format choices for optional/not-found behavior.
- Node wiring changes needed to inject storage-backed read service.
