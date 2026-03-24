# Domain Model

## Domain: RPC Memory Surface
- Owns method registration and wire schema.
- Validates hex-encoded identity fields.
- Maps service errors to RPC error objects.

## Domain: Personality Finalized Storage
- Owns canonical finalized personality entry shape (`StoredPersonality`).
- Exposes latest-by-personality-id lookup contract.

## Boundary Contract
- RPC receives strings and returns serialized response.
- Service boundary uses decoded bytes and domain struct values.
