# chainspec

## Purpose
Node-facing Sahara chain-spec ownership crate.

## Location
`crates/chainspec/`

## Owns
- `SAHARA_CHAIN_ID`
- Native-token hard-cap constants + validation helpers (`sahara_hard_cap_base_units`, `validate_genesis_alloc`, `NativeTokenError`)
- `build_sahara_chain_spec*`
- `try_build_sahara_chain_spec*`
- `try_simplex_validators_from_chain_spec`

## Dependency Boundary
- Depends on `app-evm` for fee-recipient registry constants only.
- Depends on `evm-precompiles` for epoch precompile genesis storage constants/helpers.
- Reuses `validators` codec helpers, including `encode_ethereum_address_storage_value` and validator-registry decode helpers.
- No runtime dependency from `app-evm` back to `chainspec` (only test/dev usage in `app-evm`).

## Notes
- Genesis alloc hard-cap enforcement is implemented in `chainspec::native_token` and re-exported from `chainspec` root.
- Ordered simplex-validator registry storage remains encoded/decoded through `validators`.
- Genesis builder now seeds epoch precompile state:
  - `currentEpoch=0`
  - `epochBlocks=403200`
  - `nextEpochBlock=403200`
  - `epochStartBlock(0)=0` (plus-one encoded in storage)
- Genesis builder also seeds `epoch_system_tx_sender()` balance+nonce for deterministic boundary tx execution.
- Native-token cap validation excludes the reserved epoch system sender seed balance from hard-cap summation.
