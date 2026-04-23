# integration-tests: E2E System Tests

## Summary
Workspace-level integration tests for consensus + node + RPC + EVM execution.
Location: `testing/integration-tests/`

## Dependency Boundaries
- `chainspec`: canonical Sahara chain-spec builders/chain-id used by full-node tests.
- `app-evm`: runtime-owned constants and EVM runtime behavior under test.
- `whirlpool-node`: in-process node startup via `start_node_with_chain_spec(...)`.
- `rpc-eth`, `state-reth`, `validators`, `evm-precompiles`: subsystem coverage.

## Notable Coverage
- `tests/tokenomics/native_token_supply_cap.rs`: over-cap rejection + supply conservation.
- `tests/tokenomics/community_pool.rs`: burned-fee routing, fee-pool priority-fee routing, proposer metadata behavior, fee-pool claim accrual, fee-pool withdraw end-to-end, multi-validator proposer reward routing, `Address::ZERO` no-delta guard, and precompile-vs-RPC balance parity.
- `tests/rpc/evm.rs`: JSON-RPC contract + full-node transfer/deploy/call flows.
- `tests/consensus/multinode.rs`: multi-node consensus progression.
- `src/bin/single_node_transfer_benchmark.rs`: release-binary benchmark harness; starts an in-process single node with a custom genesis (2,000 funded sender accounts by default), submits transfer traffic for a fixed 120-second window, and emits JSON metrics including block count, average block time, packaged transaction count, and TPS (= tx_count / 120s).

## Test Layout
- Cargo entrypoints: `tests/rpc_suite.rs`, `tests/tokenomics_suite.rs`, `tests/consensus_suite.rs`
- Domain modules: `tests/{rpc,tokenomics,consensus}/mod.rs`
- Shared helpers: `tests/common/http.rs`, `tests/common/encoding.rs`, and `tests/common/ports.rs`
- Benchmark support modules: `src/bin/benchmark_support/cli.rs` and `src/bin/benchmark_support/runner.rs`
- Benchmark entrypoint: `cargo run --release -p integration-tests --bin single_node_transfer_benchmark -- --duration-seconds 120 --sender-accounts 2000 --recipient-accounts 2000`

## Chain-spec Ownership Note
Integration tests source Sahara chain-spec builders, `SAHARA_CHAIN_ID`, and native-token hard-cap helpers from `chainspec`.

## Harness Notes
- RPC contract tests serialize access with an async `tokio::sync::Mutex` guard (not a blocking std mutex), so lock scope safely spans awaited RPC calls.
- Consensus multinode tests allocate ports through `tests/common/ports.rs` so repeated local binds do not recycle duplicate ephemeral ports across concurrently-started nodes.
