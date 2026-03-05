## Grounded facts
- Workspace follows interface/implementation pairs (`app`/`app-evm`, `state`/`state-memory`) plus node wiring crate (`whirlpool-node`).
- No existing RPC domain crate is present.

## [PROPOSED] deltas
- Own ETH RPC serving in node/binary domain to preserve 3-layer separation.
- Keep transport and serialization concerns out of interface trait crates.
- Maintain minimal node-local mutable indices for pending tx and receipts.
