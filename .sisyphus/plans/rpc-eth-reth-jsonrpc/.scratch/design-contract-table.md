# Design Contract Table

## Scope and Guardrails
- In scope: `crates/rpc-eth` redesign around reth JSON-RPC, plus the `crates/whirlpool-node` startup wiring needed to consume the new API.
- Out of scope: `vendor/**`, Engine/admin/debug namespaces, blob execution support, and any redesign of `state-reth`, `app-evm`, or storage ownership.
- Planning must preserve behavior-test-first sequencing, independent commit-ready checkpoints, and `nix develop --command cargo ...` gates.

## Readiness
- `agent/TASK_GEN_READY.md`: PASS.
- `agent/blockers.md`: no active blockers; only compile-time and trait-surface size concerns, both mitigated by keeping provider work sliced and mechanical.

## Canonical Crates and Roles
| Crate | Role | Planned touchpoints |
|---|---|---|
| `rpc-eth` | primary implementation crate | `Cargo.toml`, `src/lib.rs`, `src/server.rs`, new `src/provider.rs`, `src/pool.rs`, `src/network.rs`, `src/convert.rs`, tests |
| `whirlpool-node` | integration boundary | `src/main.rs` and/or `src/node.rs` updated to construct `RpcConfig` and start the new server |
| `testing/integration-tests` | end-to-end verification | `tests/rpc_integration.rs` updated to mirror reth `rpc-builder` HTTP patterns |

## Public Interface Contract
- Canonical entrypoint: `pub async fn start_rpc_server(config: RpcConfig) -> Result<RpcServerHandle, RpcError>`.
- `RpcConfig` fields: `state_db: Arc<RethStateDb>`, `tx_source: Arc<dyn TxSource>`, `chain_id: u64`, `bind_addr: SocketAddr`.
- Internal adapters: `WhirlpoolProvider`, `WhirlpoolTxPool`, `WhirlpoolNetwork`.
- Invariants:
  - REQ-1 / REQ-2 / REQ-3 / REQ-4 served through reth-backed adapters.
  - REQ-5 forbids blob execution and requires unsupported handling for `eth_blobBaseFee`.
  - REQ-6 updates `whirlpool-node` only at the wiring boundary.
  - REQ-7 requires integration tests modeled after reth HTTP RPC tests.

## Provider Trait Surface
- Real implementations required: `BlockHashReader`, `BlockNumReader`, `HeaderProvider`, `BlockReader`, `BlockReaderIdExt`, `TransactionsProvider`, `ReceiptProvider`, `StateProviderFactory`, `ChainSpecProvider`, `AccountReader`, `NodePrimitivesProvider`.
- Stub/noop implementations required first: `StageCheckpointReader`, `ChangeSetReader`, `PruneCheckpointReader`, `HashedPostStateProvider`, `StateRootProvider`, `StorageRootProvider`, `StateProofProvider`, `BlockBodyIndicesProvider`.
- `CanonStateSubscriptions` comes last in provider sequencing and uses a noop broadcast channel.

## Adapter Flow Anchors
- Server startup: `RpcModuleBuilder::new(provider, pool, network, evm, consensus).bootstrap_eth_api().build(...)`.
- Balance path: `WhirlpoolProvider::state_by_block_number` -> `RethStateDb` -> `StateDb::get_account`.
- Send raw transaction path: `EthApi::send_raw_transaction` -> `WhirlpoolTxPool::add_external` -> `TxSource`.
- Block lookup path: `WhirlpoolProvider::block_by_number` -> `BlockStorage::get_block_by_number` -> `convert::evm_block_to_reth_block`.
- Blob path: explicit unsupported method behavior plus Type-3 rejection at tx-pool ingress.

## Test Contract Mapping
- TST-1 covers provider trait completeness and builder bounds.
- TST-2 covers `WhirlpoolTxPool` bridge semantics.
- TST-3 covers `WhirlpoolNetwork` info semantics.
- TST-4..TST-11 cover HTTP startup and supported `eth_*` behavior, with blob exclusion.
- TST-12 covers `whirlpool-node` booting with the new path.

## Execution Order Contract
1. Foundation in `provider.rs`.
2. Pool and network adapters.
3. `server.rs` and `lib.rs` rewrite.
4. `convert.rs` support.
5. `whirlpool-node` integration.
6. Unit/integration tests mirroring reth patterns.

## Resolution Notes
- Use `RpcConfig` instead of the legacy positional startup signature from `agent/domains.md`.
- Standardize `eth_blobBaseFee` on unsupported behavior to satisfy REQ-5 / TST-10.
