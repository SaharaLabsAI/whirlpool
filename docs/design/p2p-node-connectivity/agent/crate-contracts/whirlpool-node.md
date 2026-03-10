# Crate Contract: whirlpool-node

## Scope
- Sub-Intent C integration boundary for `REQ-8`.
- Primary implementation remains in upstream crates:
  - `crates/consensus-simplex`
  - `crates/p2p`
  - `crates/p2p-commonware`
- `crates/whirlpool-node` is expected to consume the updated contracts with minimal or no source changes.

## Current Baseline Verified From Source
- `crates/whirlpool-node/src/main.rs` constructs `CommonwareConfig`, builds a `CommonwareNetworkProvider`, and starts `CommonwareEngine::new(...).start()`.
- The node does not manually interact with per-channel vote/certificate/resolver streams.
- The finalization path is mediated through `PersistingFinalizationSink` and `ApplicationAdapter`.

## Required Integration Outcome
- `whirlpool-node` must remain source-compatible or near-source-compatible with the relay activation changes.
- Expected node-level behavior after upstream crate updates:
  - startup still builds the network provider through `CommonwareNetworkProviderBuilder`
  - startup still constructs `CommonwareEngine`
  - `CommonwareEngine::start()` internally consumes the new payload channel pair and spawns payload persistence logic
  - the node's finalization sink and application adapter continue to behave as before

## Expected Source Change Surface
- Preferred outcome: no `crates/whirlpool-node` code change is required beyond recompiling against the updated crate contracts.
- Acceptable narrow changes if implementation demands them:
  - import updates caused by tightened generic bounds
  - test updates to reflect that multi-node relay is now active
- Not acceptable in this sub-intent:
  - node-owned payload distribution logic
  - manual payload channel registration in `main.rs`
  - redesign of RPC, state, mempool, or CLI wiring

## Compatibility Contract
- Existing startup flow in `crates/whirlpool-node/src/main.rs` remains the integration point.
- `PersistingFinalizationSink` remains untouched in semantics.
- `ApplicationAdapter` remains the app-facing block provider for consensus.
- Single-node local-dev startup remains behaviorally valid.

## Traceability
- `REQ-8` -> proves existing app-layer and node startup wiring stay compatible while relay activation becomes functional.

## Validation Expectations
- `cargo test -p whirlpool-node` must continue to pass after the relay-related crates change.
- Multi-node behavior should improve without requiring new node-specific orchestration code in this pass.
