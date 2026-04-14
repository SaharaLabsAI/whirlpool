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
- `rpc_mem_integration.rs`: currently disabled during mem unwiring transition.
- `src/bin/single_node_transfer_benchmark.rs`: release-binary benchmark harness; starts an in-process single node with a custom genesis (2,000 funded sender accounts by default), submits transfer traffic for a fixed 120-second window, and emits JSON metrics including block count, average block time, packaged transaction count, and TPS (= tx_count / 120s).

## Test Layout
- Cargo entrypoints: `tests/rpc.rs`, `tests/tokenomics.rs`, `tests/consensus.rs`
- Domain modules: `tests/{rpc,tokenomics,consensus}/mod.rs`
- Shared helpers: `tests/common/http.rs`, `tests/common/encoding.rs`, and `tests/common/ports.rs`
- Benchmark entrypoint: `cargo run --release -p integration-tests --bin single_node_transfer_benchmark -- --duration-seconds 120 --sender-accounts 2000 --recipient-accounts 2000`

## Chain-spec Ownership Note
Integration tests source Sahara chain-spec builders, `SAHARA_CHAIN_ID`, and native-token hard-cap helpers from `chainspec`.
