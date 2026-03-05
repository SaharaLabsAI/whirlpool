# AC Coverage Report

AC_VERSION verified: 2026-03-05T15:25:00+08:00

## Coverage Table

| AC ID | Criterion | Covered by Task | Status |
|-------|-----------|----------------|--------|
| AC-1 | eth_chainId returns 313371 | 03-chainid-gasprice | ✅ |
| AC-2 | eth_getBalance for known account | 04-balance-nonce | ✅ |
| AC-3 | eth_getBalance returns 0 for unknown | 04-balance-nonce | ✅ |
| AC-4 | eth_getTransactionCount returns nonce | 04-balance-nonce | ✅ |
| AC-5 | eth_estimateGas returns 21000 | 06-estimate-gas-receipt | ✅ |
| AC-6 | eth_gasPrice returns 1 gwei | 03-chainid-gasprice | ✅ |
| AC-7 | eth_sendRawTransaction returns hash | 05-send-raw-transaction | ✅ |
| AC-8 | eth_sendRawTransaction pushes to pool | 05-send-raw-transaction | ✅ |
| AC-9 | eth_getTransactionReceipt None for unknown | 06-estimate-gas-receipt | ✅ |
| AC-10 | eth_getTransactionReceipt for confirmed tx | 06-estimate-gas-receipt | ✅ |
| AC-11 | RPC server starts with consensus | 07-main-wiring-alloy-e2e | ✅ |
| AC-12 | alloy e2e balance transfer | 07-main-wiring-alloy-e2e | ✅ |

## Summary
- Total AC: 12
- Covered: 12
- Missing: 0
- Coverage: **100%**
