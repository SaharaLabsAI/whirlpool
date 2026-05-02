# validators-dkg

## Purpose
Canonical owner for DKG payload schema and DKG metadata semantics used by Whirlpool block-header `extra_data`.

## Location
`crates/validators/dkg/`

## Owns
- DKG payload structs and codecs: `FullDkgOutputV1`, `FullDkgV1`, `ReshareV1`, `encode_full_dkg_v1`, `decode_full_dkg_v1`, `encode_reshare_v1`, `decode_reshare_v1`.
- Activation handoff semantics: `EpochActivationTargets`, `ValidatorActivationSchedule`.
- DKG proposal/verify decisions: `decide_dkg_header_sections`, `validate_dkg_header_sections`; include/omit and activation-player checks are internal semantic helpers, not app-facing RawEth/header-carrier APIs.
- DKG-only error semantics: `DkgMetadataError`, `DkgPayloadError`.

## Boundary
Pure DKG semantic crate. It must not depend on `app-traits`, `app-primitives`, `state`, `app-evm-execution`, or `evm-precompiles`, and it must not own RawEth, WDX1 header envelopes, proposer extraction, RPC projection, or historical raw header scans.

## Consumers
- `app-evm-execution` supplies proposal/verify timing, active validator schedule, previous FullDkg baseline, and candidate output; it owns the historical lookup loop.
- `app-primitives` carries DKG payloads in header `DkgHeaderSections` slots and delegates only payload codec/semantic checks to this crate.
- `whirlpool-node` builds `FullDkgOutputV1` from bootstrap material.

## Verification
- `cargo test -p validators-dkg`
- Dependency gate: `validators-dkg` must remain independent from app/state/execution/precompile crates and must not expose RawEth/header-carrier helpers.
