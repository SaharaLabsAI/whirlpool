# Summary: Persistent Block Storage & History Queries

**Feature**: Persistent block storage via MDBX and history queries via eth_getBlock* RPC.

**Goal**: Enable durable storage of finalized blocks and their receipts. This allows the node to serve historical block data through Ethereum compatible RPC endpoints and reconstruct chain state after restarts.

**Architecture**:
- **Trait-first**: New `BlockStorage` trait in `state` crate defines the persistence API.
- **MDBX Backend**: `RethStateDb` implements `BlockStorage` using existing reth-db tables.
- **App-layer Hook**: Block persistence triggers in `EvmApp` during finalization events.
- **Receipt Lifecycle**: Receipts are cached during `propose` and flushed on finalization.
- **RPC Integration**: `EthRpcContext` uses the block store to serve historical queries.

**Crate Changes**:
| Crate | Change Size | Description |
|---|---|---|
| state | Minor | Added `BlockStorage` trait and result types. |
| state-reth | Moderate | Implemented `BlockStorage` using MDBX tables and conversion logic. |
| app | Minor | Re-exported `Receipt` type for trait signature consistency. |
| app-evm | Moderate | Added receipts cache and persistence hook to `EvmApp`. |
| consensus-simplex | None | No changes; persistence handled at the application layer. |
| rpc-eth | Moderate | Added `eth_getBlockByNumber` and `eth_getBlockByHash` endpoints. |
| whirlpool-node | Minor | Wired `RethStateDb` as the block storage provider for RPC. |

**Flows**:
1. Block Finalization → Persistent Storage (Atomically write header, txs, and receipts).
2. `eth_getBlockByNumber` Query (Resolve tags/numbers to MDBX reads).
3. `eth_getBlockByHash` Query (Reverse lookup via `HeaderNumbers` table).
4. Node Startup Wiring (Component assembly and shared database initialization).

**Blockers**: 0 active, 8 deferred, 3 resolved. No active blockers prevent implementation.

**Tests**: 22 unit tests, 2 integration tests, 4 flow tests. 4 UNKNOWNs identified.

**Risk Summary**:
- **Type Encoding Mismatch**: Resolved by reusing existing conversion functions.
- **Finalization Latency**: Mitigated by batched MDBX writes; deferred to perf testing.
