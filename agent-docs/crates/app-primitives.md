# app-primitives

## Purpose
Concrete app-layer block/result primitives plus canonical block-header `extra_data` carrier schema.

## Location
`crates/app/primitives/`

## Owns
- `EvmBlock`: consensus-visible EVM block carrier.
- `ExecutionResult`: execution output summary returned by app propose/verify.
- `BlockId`: `[u8; 32]` block identifier.
- `Receipt`: `alloy_consensus::Receipt` re-export for app-layer block storage and execution plumbing.
- `header_extra_data`: `CanonicalHeaderExtraDataV1`, `RawEthProposerCarrier`, `DkgHeaderSections`, WDX1 envelope encode/decode, RawEth proposer extraction, RawEth projection, exact header history read trait.

## Boundary
`app-primitives` owns the outer block-header carrier only: WDX1 magic/version, section order/limits, RawEth proposer bytes, and Ethereum-visible projection. It may carry `validators-dkg` FullDkg/Reshare payloads in typed slots, but it must not decide DKG activation, include/omit, parity, boundary validity, or candidate matching.

## Key Notes
- `EvmBlock` keeps the existing codec field order and digest/id semantics; golden tests lock encoded bytes, `compute_id()`, and `Digestible::digest()` output.
- Header `extra_data` is strict canonical envelope bytes. Raw 32-byte legacy proposer-key carriers are not valid.
- `HeaderExtraDataHistory` is a raw-byte read trait for storage adapters; historical scan orchestration lives in `app-evm-execution`.

## Canonical Imports
- `app_primitives::{EvmBlock, ExecutionResult, Receipt}`
- `app_primitives::header_extra_data::{build_header_extra_data, decode_header_extra_data, encode_header_extra_data, proposer_public_key_from_extra_data, project_raw_eth_extra_data, CanonicalHeaderExtraDataV1, DkgHeaderSections}`

## Verification
- `cargo test -p app-primitives`
