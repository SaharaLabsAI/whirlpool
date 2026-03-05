## Grounded facts
- Interface crate candidate in focus scope is `app`, exposing `Application` and `TxSource` traits with concrete tx source implementations (`crates/app/src/traits.rs`, `crates/app/src/tx_source.rs`).
- `state` is an interface crate but outside primary focus; still relevant for state reads through implementation (`crates/state/src/traits.rs`).
- Current intent does not require a new cross-crate trait to satisfy minimal RPC method implementation; node can consume existing exported handles.

## [PROPOSED] deltas
- Keep interface crates unchanged for this design set.
- If future RPC grows to multi-node consumers, consider introducing an interface crate for RPC service contracts.
