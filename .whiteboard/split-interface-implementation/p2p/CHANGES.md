# CHANGES — p2p

## Key Decisions

- **Grounded**: `p2p::traits` is already the correct canonical interface boundary.
- **[PROPOSED]**: Limit this crate to stabilization-only edits and avoid path churn.

## Current State

- `crates/p2p/src/traits.rs` already serves as trait interface surface (`PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider`).
- `crates/p2p/src/lib.rs` re-exports those traits and core message/channel types.

## Proposed Changes

- **Grounded**: Keep `crates/p2p/src/traits.rs` as canonical interface-only boundary.
- **Grounded**: Preserve stable exports in `crates/p2p/src/lib.rs` to avoid unnecessary downstream churn.
- **[PROPOSED]**: Audit and prevent concrete/vendor implementation details from entering `traits.rs` during related refactors.
- **[PROPOSED]**: If any compatibility shims are needed for adapter migration, keep them in `lib.rs`, not in `traits.rs`.

## Impact on Dependents

- `p2p-commonware` and node crates should experience no trait path breakage from this crate.
- This crate acts as a stable foundation while adapter crates (`p2p-commonware`) gain new local interfaces.

## Migration Notes

- File-level actions are expected to be minimal:
  - `crates/p2p/src/traits.rs`: no semantic change, interface-only verification.
  - `crates/p2p/src/lib.rs`: preserve current re-export contract.
- Treat this crate as a stabilization checkpoint in migration sequencing.
