# validators-dkg

## Purpose
Canonical owner for DKG metadata/schema semantics used by Whirlpool block `extra_data`.

## Location
`crates/validators/dkg/`

## Owns
- DKG-aware canonical extra-data envelope: magic/version, section order, limits, decode modes, errors.
- `CanonicalExtraDataV1`, `FullDkgOutputV1`, `FullDkgV1`, `ReshareV1`.
- Codec/projection helpers: `encode_canonical_extra_data`, `decode_extra_data`, `legacy_proposer_extra_data_bytes`, `project_raw_eth_extra_data`, `proposer_public_key_from_extra_data`.
- Activation handoff semantics: `EpochActivationTargets`, `ValidatorActivationSchedule`, `BoundaryValidatorActivation`.
- DKG proposal and verify helpers: `build_canonical_dkg_extra_data`, `validate_dkg_extra_data`, `full_dkg_should_be_included`, `ensure_full_dkg_players_match_activation`.
- Historical scan algorithm over `DkgHistory::full_dkg_at_height`, which retrieves raw carrier bytes and decodes them inside `validators-dkg`.

## Boundary
Pure semantic crate. It must not depend on `app`, `state`, `app-evm-execution`, or `evm-precompiles`. Runtime crates adapt local storage/config into DKG-owned input structs and traits.

## Consumers
- `app-evm-execution` supplies proposal/verify timing and config values; storage crates implement `DkgHistory` for raw historical carrier-byte lookup.
- `app-evm-state` and `rpc-eth` use projection helpers for block storage/RPC compatibility.
- `whirlpool-node` builds `FullDkgOutputV1` from bootstrap material.

## Verification
- `cargo test -p validators-dkg`
- Dependency gate: `validators-dkg` must remain independent from app/state/execution/precompile crates.
