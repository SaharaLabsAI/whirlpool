# chainspec

## Purpose
Node-facing Sahara chain-spec ownership crate.

## Location
`crates/chainspec/`

## Owns
- `SAHARA_CHAIN_ID`
- `build_sahara_chain_spec*`
- `try_build_sahara_chain_spec*`
- `try_simplex_validators_from_chain_spec`

## Dependency Boundary
- Depends on `app-evm` for fee-recipient registry constants only.
- Reuses `validators` codec helpers, including `encode_ethereum_address_storage_value` and validator-registry decode helpers.
- No runtime dependency from `app-evm` back to `chainspec` (only test/dev usage in `app-evm`).

## Notes
- Genesis alloc hard-cap enforcement remains via `native-token::validate_genesis_alloc`.
- Ordered simplex-validator registry storage remains encoded/decoded through `validators`.
