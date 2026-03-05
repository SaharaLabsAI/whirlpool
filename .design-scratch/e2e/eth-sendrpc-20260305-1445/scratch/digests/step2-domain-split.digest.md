## Grounded facts
- Workspace already uses interface/implementation split in multiple domains (`app`/`app-evm`, `state`/`state-memory`).
- Intent introduces RPC serving capability but targeted to node runtime behavior, not a reusable domain interface consumed by multiple implementation crates today.

## [PROPOSED] deltas
- Do not auto-propose a new interface/impl crate pair for this intent.
- Place RPC modules under `whirlpool-node` with explicit extension path documented in strategy if reuse emerges.
