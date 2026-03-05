# Proven Acceptance Criteria

AC_VERSION: 2026-03-05T15:25:00+08:00

## Acceptance Criteria

| ID | Criterion | Verification | Evidence |
|----|-----------|-------------|----------|
| AC-1 | eth_chainId returns U64(313371) | TC-001 | SAHARA_CHAIN_ID (app-evm/src/config.rs) |
| AC-2 | eth_getBalance returns U256 for known account | TC-002 | StateDb::get_account (state/src/traits.rs) |
| AC-3 | eth_getBalance returns U256::ZERO for unknown account | TC-002 | StateDb::get_account returns None |
| AC-4 | eth_getTransactionCount returns nonce as U256 | TC-003 | StateDb::get_account (state/src/traits.rs) |
| AC-5 | eth_estimateGas returns U256(21000) for transfer | TC-004 | Design: hardcoded v1 (STRATEGY.md) |
| AC-6 | eth_gasPrice returns U256(1_000_000_000) | TC-005 | Design: hardcoded v1 (STRATEGY.md) |
| AC-7 | eth_sendRawTransaction accepts valid tx, returns B256 hash | TC-006 | InMemoryTxPool::push (app/src/tx_source.rs) |
| AC-8 | eth_sendRawTransaction pushes tx bytes to pool | TC-006 | InMemoryTxPool (app/src/tx_source.rs) |
| AC-9 | eth_getTransactionReceipt returns None for unknown hash | TC-007 | [PROPOSED] ReceiptStore |
| AC-10 | eth_getTransactionReceipt returns receipt for confirmed tx | TC-008 | [PROPOSED] ReceiptStore |
| AC-11 | RPC server starts alongside consensus engine | TC-009 | Node wiring (WORKSPACE.md) |
| AC-12 | alloy client can send ETH transfer and verify balance change | TC-010 | All methods combined (FLOWS.md) |

## QA Scenarios

| ID | Scenario | Expected |
|----|----------|----------|
| QA-1 | Port already in use | Server fails, node continues consensus |
| QA-2 | Invalid RLP bytes to sendRawTransaction | JSON-RPC error -32602 |
| QA-3 | Invalid address format | JSON-RPC error -32602 |
| QA-4 | Concurrent RPC during block production | All succeed |
| QA-5 | getTransactionReceipt before finalization | Returns None |

## Invariants

| ID | Statement | Evidence |
|----|-----------|----------|
| INV-1 | RPC never modifies consensus state directly | Boundary: writes only to InMemoryTxPool |
| INV-2 | State reads are always consistent | RwLock guarantees |
| INV-3 | tx_pool.push is ONLY ingress for RPC txs | Single write method |
| INV-4 | Receipt store is append-only | Insert only during execution |
| INV-5 | Chain ID is immutable | Const value |
