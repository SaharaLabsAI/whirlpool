# validators

## Purpose
Thin compatibility/query facade for the canonical Whirlpool validator registry model.

## Location
`crates/validators/`

## Ownership Boundary
Canonical validator semantics now live in `evm-precompiles::validators`. This crate exists so downstream consumers such as chainspec, node/RPC code, and integration tests can continue importing `validators::{...}` without making `evm-precompiles` depend on `validators`.

## Key exports
All exports are re-exports from `evm_precompiles::validators`:
- `ValidatorEntry { consensus_pubkey, ethereum_address }`
- `SIMPLEX_VALIDATORS_REGISTRY`
- `encode_validator_registry_storage(entries)`
- `decode_validator_registry_storage(storage)`
- `decode_validator_registry_storage_opt(storage)`
- `ordered_consensus_pubkeys(entries)`
- `encode_ethereum_address_storage_value(address)`
- `ValidatorRegistryError`

## Internal structure
- `src/lib.rs`: re-export/forward-only public facade plus wrapper-compatibility tests.
- No local registry codec/address-storage implementation files remain.

## Guarantees
- Public Rust callers using the `validators` crate keep the same import surface.
- Registry codec, storage layout, and validator entry semantics are not duplicated here.
- Allowed dependency direction is `validators -> evm-precompiles`; `evm-precompiles -> validators` is forbidden.

## Status
Compatibility facade. Do not add activation logic, registry slot math, address-padding logic, or validator semantic rules here; add those under `evm-precompiles::validators`.
