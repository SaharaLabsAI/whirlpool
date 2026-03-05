## Grounded facts
- Current workspace execution types use reth/alloy primitives in app-evm (`crates/app-evm/src/executor.rs`).
- JSON-RPC layer is currently absent, so no existing transport type layer conflict exists.

## [PROPOSED] deltas
- Target the high-level JSON-RPC method type layer expected by alloy provider clients.
- Keep internal execution types (recovered tx, bundle state internals) hidden behind node-local handler/context boundaries.
- Map request bytes -> tx hash / pool insert / receipt lookup through explicit adapter functions in node-local RPC module.
