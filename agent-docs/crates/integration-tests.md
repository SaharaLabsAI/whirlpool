# integration-tests: E2E System Tests

## Summary
Workspace-level integration tests for consensus + node + RPC + EVM execution.
Location: `testing/integration-tests/`

## Dependency Boundaries
- `chainspec`: canonical Sahara chain-spec builders/chain-id used by full-node tests.
- `app-evm`: runtime-owned constants and EVM runtime behavior under test.
- `whirlpool-node`: in-process node startup via `start_node_with_chain_spec(...)`.
- `rpc-eth`, `state-reth`, `native-token`, `validators`, `community-pool`: subsystem coverage.

## Notable Coverage
- `native_token_supply_cap.rs`: over-cap rejection + supply conservation.
- `community_pool.rs`: burned-fee routing and proposer priority-fee recipient behavior.
- `rpc_evm_integration.rs`: JSON-RPC contract + full-node transfer/deploy/call flows.
- `precompile_test_token.rs`: custom precompile end-to-end behavior.
- `multinode_consensus.rs`: multi-node consensus progression.
- `rpc_mem_integration.rs`: currently disabled during mem unwiring transition.

## Chain-spec Ownership Note
Integration tests now source Sahara chain-spec builders and `SAHARA_CHAIN_ID` from `chainspec` instead of `app-evm`.
