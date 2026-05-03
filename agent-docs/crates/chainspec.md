# chainspec

## Purpose
Node-facing Sahara chain-spec ownership crate.

## Location
`crates/chainspec/`

## Owns
- `SAHARA_CHAIN_ID`
- `genesis::SaharaGenesisConfig`
- `genesis::build_sahara_chain_spec()`
- `genesis::build_sahara_chain_spec_from(SaharaGenesisConfig)`
- `genesis::try_build_sahara_chain_spec_from(SaharaGenesisConfig)`
- Native-token hard-cap constants + validation helpers under `native_token`
- `community_pool::CommunityPoolUnlockConfig`
- `validators::try_simplex_validators_from_chain_spec`

## Dependency Boundary
- Does not own proposer fee-recipient mapping; validator `ethereum_address` values are seeded only through `SIMPLEX_VALIDATORS_REGISTRY`.
- Depends on `evm-precompiles` for epoch + community-pool unlock storage constants/helpers.
- Reuses `validators-reader` codec helpers for validator-registry genesis storage.
- Does not depend on `app-evm-execution`; runtime execution remains outside the chain-spec ownership boundary.

## Module Taxonomy
- `src/lib.rs` is a semantic domain map plus `SAHARA_CHAIN_ID`; it does not re-export behavioral builder overloads.
- `src/genesis/mod.rs` owns public chain-spec construction and typed passive genesis inputs.
- `src/genesis/storage.rs` owns private genesis storage seeding for validator registry, epoch precompile, epoch system sender, and community-pool unlock state.
- `src/community_pool.rs` owns `CommunityPoolUnlockConfig`.
- `src/native_token.rs` owns native-token cap constants, supply summation, and genesis alloc validation.
- `src/validators.rs` owns chain-spec validator-registry readback.

## Notes
- Public chain-spec construction is module-qualified: use `chainspec::genesis::*`, not root builder functions.
- `SaharaGenesisConfig` has passive fields: `alloc`, `simplex_validators`, and `community_pool_unlock`.
- Default construction uses empty alloc, no simplex validators, and disabled community-pool unlock config.
- Ordered simplex-validator registry storage remains encoded/decoded through `validators-reader`; separate fee-recipient genesis-map builders were removed.
- Genesis builder seeds epoch precompile state:
  - `currentEpoch=0`
  - `epochBlocks=403200`
  - `nextEpochBlock=403200`
  - `epochStartBlock(0)=0` (plus-one encoded in storage)
- Genesis builder seeds `epoch_system_tx_sender()` balance+nonce for deterministic boundary tx execution.
- Optional community-pool genesis seeding includes:
  - prefund at `COMMUNITY_POOL_ADDRESS`
  - unlock cadence slots (`unlockEveryEpochs`, `unlockAmountPerCycle`)
  - unlock progress slots (`lockedRemaining`, `lastProcessedEpoch`)
- Unlock enablement guard: if unlock schedule is enabled, `simplex_validators` must be non-empty.
- Native-token cap validation excludes the reserved epoch system sender seed balance from hard-cap summation.
- Native-token unit coverage lives in source-adjacent `src/native_token_tests.rs`; genesis behavior coverage lives in `src/tests.rs`.
