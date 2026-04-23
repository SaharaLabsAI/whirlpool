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
- `CommunityPoolUnlockConfig` and the extended builder path `*_and_community_pool_unlock_config`

## Dependency Boundary
- Depends on `app-evm-execution` for fee-recipient registry constants only.
- Depends on `evm-precompiles` for epoch + community-pool unlock storage constants/helpers.
- Reuses `validators` codec helpers, including `encode_ethereum_address_storage_value` and validator-registry decode helpers.
- No runtime dependency from `app-evm-execution` back to `chainspec` (only test/dev usage in `app-evm-execution`).

## Notes
- Root `chainspec` API is now a thin re-export surface in `src/lib.rs`; builder logic is split across focused modules (`spec_builders_base`, `spec_builders_alloc`, `spec_builders_try`, `spec_builders_core`) to keep per-file cohesion policy limits satisfied without changing public signatures.
- Validator-registry readback is isolated in `src/simplex_validator_reader.rs` and re-exported as `try_simplex_validators_from_chain_spec`.
- `CommunityPoolUnlockConfig` now lives in `src/community_pool_unlock.rs` and remains re-exported from crate root.
- Genesis alloc hard-cap enforcement is implemented in `chainspec::native_token` and re-exported from `chainspec` root.
- Ordered simplex-validator registry storage remains encoded/decoded through `validators`.
- Genesis builder now seeds epoch precompile state:
  - `currentEpoch=0`
  - `epochBlocks=403200`
  - `nextEpochBlock=403200`
  - `epochStartBlock(0)=0` (plus-one encoded in storage)
- Genesis builder also seeds `epoch_system_tx_sender()` balance+nonce for deterministic boundary tx execution.
- Optional community-pool genesis seeding now includes:
  - prefund at `COMMUNITY_POOL_ADDRESS`
  - unlock cadence slots (`unlockEveryEpochs`, `unlockAmountPerCycle`)
  - unlock progress slots (`lockedRemaining`, `lastProcessedEpoch`)
- Unlock enablement guard: if unlock schedule is enabled, `simplex_validators` must be non-empty.
- Native-token cap validation excludes the reserved epoch system sender seed balance from hard-cap summation.
