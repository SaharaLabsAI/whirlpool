# Crate Contract: rpc-mem

## Responsibilities
- Register `mem_submitPersonality` and `mem_getPersonality` methods.
- Validate read request fields (`personality_id`) as 0x-prefixed hex.
- Return deterministic response mapping from finalized storage model.

## Inputs
- Submit request payload.
- Get request with personality ID.

## Outputs
- Submit hash response.
- Optional personality payload response.

## Error Contract
- Malformed hex inputs map to rpc-mem validation errors.
- Storage/service failures map to rpc-mem internal/service errors.
