# integration-tests: E2E System Tests

## Summary
Workspace-level integration tests that exercise the full Whirlpool stack (consensus + p2p + RPC + state + EVM).
This crate contains no library or binary — only integration tests under `tests/`.

Location: `testing/integration-tests/`

## Dependency Boundaries
- `app`: application traits and tx source (`InMemoryTxPool`).
- `app-evm`: EVM application, `build_sahara_chain_spec()` / `build_sahara_chain_spec_with_alloc()` for chain spec construction.
- `rpc-eth`: `RpcConfig`, `start_rpc_server` — Ethereum JSON-RPC server wiring.
- `state-reth`: `RethStateDb`, `open_state_db()` — persistent MDBX state for RPC tests.
- `whirlpool-node`: `start_node_with_chain_spec()` — in-process node for full-node tests.
- `reth-rpc-builder`: `RpcServerHandle` type (test server lifecycle).
- `reth-chainspec`: `ChainSpec` type for custom chain specs.
- `alloy-primitives`: Ethereum types (Address, B256, U256).
- `alloy-genesis`: `GenesisAccount` for pre-funded genesis allocations.
- `serde_json`: JSON construction for raw HTTP RPC calls.
- `reqwest`: HTTP client for raw RPC calls.
- `tempfile`: `TempDir` for ephemeral MDBX databases.
- `tokio`: async runtime with test-util.

## Test Files

### `tests/community_pool.rs`
Full-node fee-accounting coverage for the new community-pool slice.
- Reuses the `rpc_evm_integration.rs` patterns for ephemeral ports, funded node startup, JSON-RPC helpers, EIP-1559 signing, block polling, and receipt polling, but keeps helpers local to this file.
- Uses a zero-value EIP-1559 transfer with nonzero priority fee to isolate fee accounting from value transfer.
- Fetches the exact finalized block referenced by the receipt's `blockNumber` and asserts the transaction hash is present before using block fee fields.

Tests:
- `test_community_pool_accrues_burned_amount_from_fee_only_transfer`: verifies `COMMUNITY_POOL_ADDRESS` balance delta equals `block.gasUsed * block.baseFeePerGas` for the finalized tx block.
- `test_proposer_fee_recipient_accrues_priority_fee_from_fee_only_transfer`: starts a single node whose validator fee recipient is fixed in genesis, then verifies the configured recipient gets the tx block's priority-fee remainder while `DEFAULT_PROPOSER_FEE_RECIPIENT` gets zero.
- `test_multivalidator_priority_fee_follows_actual_proposer`: starts a multi-validator network with distinct genesis-configured fee-recipient addresses, broadcasts one EIP-1559 tx to all nodes, and verifies the rewarded block's `miner`/beneficiary matches the actual proposer's configured recipient while non-proposers and the legacy hardcoded recipient get zero.

### `tests/rpc_evm_integration.rs`
Uses a shared test harness:
- `start_test_rpc() -> TestRpcServer`: Creates ephemeral `TempDir` + `RethStateDb` + `ChainSpec` via `build_sahara_chain_spec()`, wires through `RpcConfig` + `start_rpc_server()`. Returns handle, address, and temp dir.
- `RecordingTxSource`: Test `TxSource` implementation that records submitted transactions for assertion.
- `rpc_call(addr, method, params) -> serde_json::Value`: HTTP POST helper for raw JSON-RPC calls.

**Smoke tests (8 total, `tst4`–`tst11`):**
- `tst4_server_returns_chain_id` (TST-4/5): Verifies `eth_chainId` returns Sahara chain ID (`0x313532363334`).
- `tst5_latest_block_number` (TST-6): Verifies `eth_blockNumber` returns `0x0` on empty DB.
- `tst6_balance_query_returns_zero_for_empty_db` (TST-7): Verifies `eth_getBalance` returns `0x0` for unknown address.
- `tst7_eth_syncing_returns_false` (TST-7/12): Verifies `eth_syncing` returns `false`.
- `tst8_get_block_by_number` (TST-8): Verifies `eth_getBlockByNumber("0x0", false)` returns `null` on empty DB.
- `tst9_send_raw_transaction_acceptance_and_blob_rejection` (TST-9): Sends valid EIP-1559 tx (accepted) and blob Type-3 tx (rejected with pool error).
- `tst10_blob_base_fee_behavior` (TST-10): Verifies `eth_blobBaseFee` returns method-not-found or error (blob support excluded).
- `tst11_request_shape_permutations` (TST-11): Mirrors reth rpc-builder test patterns — tests HTTP request/response shape for multiple methods in a single test.

**Contract tests (30 total, `contract_*`):**
Per-method param validation tests mirroring reth `rpc-builder/tests/it/http.rs`. Each test calls one JSON-RPC method with valid params (expect success or well-formed null on empty DB) and with invalid/missing params (expect JSON-RPC error). Uses shared helpers: `assert_rpc_ok()`, `assert_rpc_err()`, `rpc_req()`.

*Address + block param:*
- `contract_eth_get_transaction_count`: ok with address+block, ok with address only, err with no params, err with bad address.
- `contract_eth_get_code`: ok with address+block, err with no params, err with bad address.
- `contract_eth_get_storage_at`: ok with address+slot+block, err with no params, err with bad address.

*Hash param:*
- `contract_eth_get_block_by_hash`: ok (returns null for unknown hash), err with no params, err with bad hash.
- `contract_eth_get_transaction_by_hash`: ok (returns null), err with no params, err with bad hash.
- `contract_eth_get_transaction_receipt`: ok (returns null), err with no params, err with bad hash.
- `contract_eth_get_block_transaction_count_by_hash`: ok, err with no params, err with bad hash.
- `contract_eth_get_uncle_count_by_block_hash`: ok, err with no params.

*Number param:*
- `contract_eth_get_block_transaction_count_by_number`: ok with `0x0`, err with no params.
- `contract_eth_get_uncle_count_by_block_number`: ok with `0x0`, err with no params.
- `contract_eth_get_block_receipts`: ok with `0x0`, err with no params.

*Index methods (block + index):*
- `contract_eth_get_uncle_by_block_hash_and_index`: ok, err with no params.
- `contract_eth_get_uncle_by_block_number_and_index`: ok, err with no params.
- `contract_eth_get_transaction_by_block_hash_and_index`: ok, err with no params.
- `contract_eth_get_transaction_by_block_number_and_index`: ok, err with no params.

*Fee / estimate / call (expect error on empty DB):*
- `contract_eth_fee_history`: accept success or error on empty DB, err with no params, err with bad count.
- `contract_eth_estimate_gas`: err on empty DB, err with no params.
- `contract_eth_call`: err on empty DB, err with no params.
- `contract_eth_create_access_list`: err on empty DB, err with no params.
- `contract_eth_max_priority_fee_per_gas`: accept error or hex on empty DB.

*Net / Web3:*
- `contract_net_version`: ok.
- `contract_net_peer_count`: ok.
- `contract_net_listening`: ok.
- `contract_web3_client_version`: ok.
- `contract_web3_sha3`: ok with valid hex (verifies keccak256 output), err with no params, err with bad hex.

*Unimplemented / protocol:*
- `contract_eth_coinbase`: err (unimplemented).
- `contract_eth_mining`: accept error or false.
- `contract_eth_get_work`: err (unimplemented).
- `contract_eth_submit_work`: err (unimplemented).
- `contract_eth_protocol_version`: accept success or error.

**Excluded (blob tx):** No tests for `eth_sendTransaction` with type-3 blobs, `eth_blobBaseFee` param variants, or any EIP-4844-specific methods beyond what `tst10` already covers.

**Full-node transaction tests (3 total, `test_*_full_node`):**
Boot an in-process whirlpool node with pre-funded genesis accounts, submit real transactions, wait for block inclusion, and verify state changes.

Shared helpers:
- `allocate_port() -> u16`: Finds an available TCP port via ephemeral binding.
- `start_funded_node(seed, funded_addresses) -> (NodeHandle, SocketAddr)`: Builds a custom chain spec with pre-funded accounts (100 ETH each), starts a node via `start_node_with_chain_spec()`.
- `wait_for_block(addr, min_height, timeout)`: Polls `eth_blockNumber` until target height reached.
- `wait_for_receipt(addr, tx_hash, timeout) -> serde_json::Value`: Polls `eth_getTransactionReceipt` until non-null.
- `send_raw_tx(addr, raw_hex) -> String`: Submits via `eth_sendRawTransaction`, returns tx hash.
- `sign_eip1559_tx(key, nonce, to, value, data, gas_limit) -> (TxHash, raw_hex)`: Signs an EIP-1559 transaction with chain_id=313371, max_fee=20gwei, max_priority_fee=1gwei.
- `deploy_minimal_contract(addr, key, nonce) -> (TxHash, raw_hex)`: Deploys a minimal contract that returns `uint256(42)`. Init code: 21 bytes, runtime: 10 bytes.

Tests:
- `test_eth_transfer_full_node` (seed=100): Fund sender with 100 ETH → send 1 ETH to recipient → verify receipt status=1 → verify recipient balance via `eth_getBalance`.
- `test_contract_deploy_full_node` (seed=101): Deploy minimal contract → verify receipt status=1, `contractAddress` present → verify deployed code via `eth_getCode` → verify `eth_call` returns 42.
- `test_contract_call_full_node` (seed=102): Deploy contract → call `eth_call` against deployed address → verify return value is `uint256(42)` (ABI-encoded).

### `tests/multinode_consensus.rs`
End-to-end consensus test for a 4-node network using `whirlpool_node::node::start_node`.
- **TC-INT-05**: Verifies multi-node consensus. Starts 4 in-process nodes, waits for block height >= 1 via `eth_blockNumber` RPC calls (testing/integration-tests/tests/multinode_consensus.rs:19).
- Uses distinct ephemeral data directories, P2P ports, and RPC ports for each node.
- Validates that heights across nodes remain synchronized within 1 block.

### `tests/rpc_mem_integration.rs`
Full-node mem RPC coverage using `whirlpool_node::node::start_node_with_chain_spec` with per-test tempdirs and dedicated mem RPC ports.
- `test_mem_submit_personality_on_mem_rpc_only`: verifies `mem_submitPersonality` succeeds on `NodeHandle::mem_rpc_addr` and is absent from the Ethereum RPC server.
- `test_mem_get_personality_returns_null_when_missing`: verifies `mem_getPersonality` returns JSON `null` for unknown IDs on the mem RPC server and is absent from the Ethereum RPC server.
- `test_mem_get_personality_returns_finalized_entry_after_submit`: submits a personality tx, waits for finalization, then verifies `mem_getPersonality` returns the finalized entry with deterministic `tx_hash`, `markdown_hash`, and stored fields.
- `test_mem_get_transaction_by_hash_returns_null_when_missing`: verifies `mem_getTransactionByHash` returns JSON `null` for unknown tx hashes and is absent from the Ethereum RPC server.
- `test_mem_get_transaction_by_hash_rejects_malformed_hash`: verifies malformed `tx_hash` input is rejected on the mem RPC server.
- `test_mem_get_transaction_by_hash_returns_finalized_entry_after_submit`: submits a personality tx, waits for finalization, then verifies `mem_getTransactionByHash` returns decoded tx fields (`version`, `signature_scheme`, `signature`) plus finalized metadata.

## Running
```bash
# Run only EVM RPC integration tests
nix develop --command cargo test -p integration-tests --test rpc_evm_integration

# Run only community-pool integration tests
nix develop --command cargo test -p integration-tests --test community_pool

# Run only mem RPC integration tests
nix develop --command cargo test -p integration-tests --test rpc_mem_integration

# Run only consensus integration tests
nix develop --command cargo test -p integration-tests --test multinode_consensus

# Run all integration tests
nix develop --command cargo test -p integration-tests

# Run workspace tests excluding integration tests (fast iteration)
nix develop --command cargo test --workspace --exclude integration-tests
```
