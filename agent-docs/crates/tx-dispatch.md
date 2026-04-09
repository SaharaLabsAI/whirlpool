# tx-dispatch

## Purpose
Mem-scoped transaction classification for mixed-ingress code.

## Dependency Boundaries
- `app-evm`: canonical EVM transaction decode and signer recovery helpers.
- `app-mem`: mem/personality transaction validation.

## Public API
- `RecoveredTx`: recovered EVM transaction type alias.
- `ClassifiedTransaction`
  - `Evm(Vec<u8>)`
  - `Mem(Vec<u8>)`
- `TxDispatchError`
- `decode_evm_transaction(raw_tx)`
- `decode_evm_transactions(raw_txs)`
- `classify_transaction(raw_tx)`
- `classify_transactions(raw_txs)`

## Classification Rule
1. Try the `app-evm` EVM decode/recovery path.
2. If that fails, try mem/personality decode.
3. If both fail, reject the transaction as invalid for the mixed domain.

## Status
Active. This crate now lives under `crates/app/execute/mem/` and remains the mixed classifier for `app-composite`; `app-evm` no longer depends on it.
