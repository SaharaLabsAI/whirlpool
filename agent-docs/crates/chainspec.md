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
- Does not own proposer fee-recipient mapping; validator `ethereum_address` values are seeded only through `SIMPLEX_VALIDATORS_REGISTRY`.
- Depends on `evm-precompiles` for epoch + community-pool unlock storage constants/helpers.
- Reuses `validators-reader` codec helpers for validator-registry genesis storage.
- No runtime dependency from `app-evm-execution` back to `chainspec` (only test/dev usage in `app-evm-execution`).

## Notes
- Root `chainspec` API is now a thin re-export surface in `src/lib.rs`; builder logic is split across focused modules (`spec_builders_base`, `spec_builders_alloc`, `spec_builders_try`, `spec_builders_core`) to keep per-file cohesion policy limits satisfied.
- Validator-registry readback is isolated in `src/simplex_validator_reader.rs` and re-exported as `try_simplex_validators_from_chain_spec`.
- `CommunityPoolUnlockConfig` now lives in `src/community_pool_unlock.rs` and remains re-exported from crate root.
- Genesis alloc hard-cap enforcement is implemented in `chainspec::native_token` and re-exported from `chainspec` root.
- Ordered simplex-validator registry storage remains encoded/decoded through `validators-reader`; separate fee-recipient genesis-map builders were removed.
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
