# validators

## Purpose
Canonical ordered simplex validator registry model and genesis-storage codec shared across `app-evm`, `evm-precompiles`, and `whirlpool-node`.

## Location
`crates/validators/`

## Key exports
- `ValidatorEntry { consensus_pubkey, ethereum_address }`
- `SIMPLEX_VALIDATORS_REGISTRY`: dedicated genesis account for ordered simplex validators.
- `encode_validator_registry_storage(entries)`
- `decode_validator_registry_storage(storage)`
- `decode_validator_registry_storage_opt(storage)`
- `ordered_consensus_pubkeys(entries)`

## Storage layout
- Slot `0`: validator count.
- For index `i`:
  - slot `2*i + 1`: `consensus_pubkey` (`bytes32`)
  - slot `2*i + 2`: `ethereum_address` (left-padded `address` in `bytes32`)

## Guarantees
- Round-trip decode preserves caller-supplied order.
- Empty/missing registry decodes to an empty list.
- Invalid address padding is rejected.

## Status
Active. This crate is the single source of truth for validator-registry encoding/decoding semantics.
