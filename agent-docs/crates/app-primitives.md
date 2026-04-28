# app-primitives

## Purpose
Concrete app-layer block/result primitives and carrier-only header `extra_data` helpers.

## Location
`crates/app/primitives/`

## Owns
- `EvmBlock`: consensus-visible EVM block carrier.
- `ExecutionResult`: execution output summary returned by app propose/verify.
- `BlockId`: `[u8; 32]` block identifier.
- `Receipt`: `alloy_consensus::Receipt` re-export for app-layer block storage and execution plumbing.
- `header_extra_data`: carrier-only helpers for strict canonical decode, RawEth proposer extraction, RawEth projection, and canonical RawEth envelope construction.

## Boundary
`app-primitives` owns carrier adaptation only. DKG schema, FullDKG/Reshare validation, activation scheduling, include/omit policy, historical scans, and semantic errors remain in `validators-dkg`.

## Key Notes
- `EvmBlock` keeps the existing codec field order and digest/id semantics; golden tests lock encoded bytes, `compute_id()`, and `Digestible::digest()` output.
- Header `extra_data` is strict canonical envelope bytes. Raw 32-byte legacy proposer-key carriers are not valid.

## Canonical Imports
- `app_primitives::{EvmBlock, ExecutionResult, Receipt}`
- `app_primitives::header_extra_data::{build_raw_eth_envelope, decode_strict_extra_data, proposer_public_key_from_extra_data, project_raw_eth_extra_data}`

## Verification
- `cargo test -p app-primitives`
