# TESTS

## Test contracts

| TestID | Proposed Name | Intent | Expected File | Owner Task | Binding |
|--------|--------------|--------|---------------|------------|---------|
| TC-001 | test_eth_chain_id_returns_sahara_id | Verify `eth_chainId` returns configured Sahara chain id for signing compatibility. | `crates/whirlpool-node/tests/rpc_chain_id.rs` | — | No |
| TC-002 | test_eth_get_balance_latest_reads_state_db | Verify `eth_getBalance` reads account balance from node state DB at `latest`. | `crates/whirlpool-node/tests/rpc_balance.rs` | — | No |
| TC-003 | test_eth_get_tx_count_latest_reads_account_nonce | Verify `eth_getTransactionCount` returns account nonce from canonical state. | `crates/whirlpool-node/tests/rpc_nonce.rs` | — | No |
| TC-004 | test_eth_estimate_gas_simple_transfer | Verify `eth_estimateGas` returns stable estimate for basic transfer request. | `crates/whirlpool-node/tests/rpc_estimate_gas.rs` | — | No |
| TC-005 | test_eth_gas_price_returns_configured_value | Verify `eth_gasPrice` returns deterministic dev-chain gas price value. | `crates/whirlpool-node/tests/rpc_gas_price.rs` | — | No |
| TC-006 | test_eth_send_raw_transaction_pushes_pool_and_returns_hash | Verify `eth_sendRawTransaction` inserts raw tx bytes into shared pool and returns tx hash. | `crates/whirlpool-node/tests/rpc_send_raw.rs` | — | No |
| TC-007 | test_eth_get_receipt_returns_none_for_unknown_hash | Verify receipt polling returns `None` for unknown or unconfirmed transaction hash. | `crates/whirlpool-node/tests/rpc_receipt.rs` | — | No |
| TC-008 | test_eth_get_receipt_returns_confirmed_receipt_after_proposal | Verify receipt becomes available after tx is executed/confirmed in proposal cycle. | `crates/whirlpool-node/tests/rpc_receipt.rs` | — | No |
| TC-009 | test_transfer_e2e_alloy_provider_send_and_confirm | Verify alloy client can sign/send raw transfer, poll receipt, and observe confirmation. | `crates/whirlpool-node/tests/rpc_alloy_transfer_e2e.rs` | — | No |
| TC-010 | test_balance_delta_after_confirmed_transfer | Verify sender/receiver balance deltas after confirmed transfer match expected value + fees. | `crates/whirlpool-node/tests/rpc_alloy_transfer_e2e.rs` | — | No |

## Coverage mapping to intent success criteria
- SC-01 -> TC-001..TC-008
- SC-02 -> TC-009
- SC-03 -> TC-006
- SC-04 -> TC-002, TC-003
- SC-05 -> TC-004
- SC-06 -> TC-007, TC-008
- SC-07 -> TC-009, TC-010

## Test Scaffolding Patterns
- Prefer black-box RPC tests that boot node runtime once per test module and call methods via HTTP client.
- Use alloy provider for end-to-end tests (`TC-009`, `TC-010`) to validate real client expectations.
- Seed deterministic accounts/genesis balances so balance and nonce assertions remain stable.
- Keep receipt polling bounded with timeout/retry loops; assert both pending and confirmed phases.
