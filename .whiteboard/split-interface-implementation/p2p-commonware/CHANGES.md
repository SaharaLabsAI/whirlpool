# CHANGES — p2p-commonware

## Key Decisions

- **Grounded**: Add an explicit local transport interface boundary.
- **[PROPOSED]**: Introduce `CommonwareTransport` additively without changing existing provider semantics.

## Current State

- `crates/p2p-commonware/src/lib.rs` and `provider.rs` concentrate transport/provider behavior without an explicit local transport trait boundary.
- Multiplex sender/receiver and provider setup are coupled to concrete commonware discovery types.

## Proposed Changes

- **Grounded**: Introduce additive `CommonwareTransport` interface in `crates/p2p-commonware/src/traits.rs`.
- **[PROPOSED]**: Define transport-level contract separately from concrete `CommonwareNetworkProvider`, `MultiplexSender`, and `MultiplexReceiver`.
- **[PROPOSED]**: Implement trait for the appropriate transport/provider abstraction without changing existing associated type signatures in `NetworkProvider` impls.
- **[PROPOSED]**: Export new trait from `crates/p2p-commonware/src/lib.rs` while preserving current public API and constructors.

## Impact on Dependents

- `consensus-simplex` and node wiring gain a clearer transport contract without immediate breakage.
- Existing provider-based integration continues working through compatibility exports and unchanged provider methods.

## Migration Notes

- File-level edits:
  - `crates/p2p-commonware/src/traits.rs`: define `CommonwareTransport`.
  - `crates/p2p-commonware/src/lib.rs`: expose `pub mod traits;` and trait export.
  - `crates/p2p-commonware/src/provider.rs` (or focused impl module): implement/adopt transport trait.
- Keep dependency direction acyclic (`p2p-commonware` remains adapter over `p2p`).
