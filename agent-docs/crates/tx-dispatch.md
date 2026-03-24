# tx-dispatch

## Purpose
Neutral transaction classification and decode helpers shared by mixed-ingress code.

## Dependency Boundaries
- `app-mem`: mem/personality transaction validation.
- `reth-ethereum-primitives` + `reth-primitives-traits`: EVM signed transaction decoding and signer recovery.

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
1. Try EVM `2718` decode and signer recovery.
2. If that fails, try mem/personality decode.
3. If both fail, reject the transaction as invalid for the mixed domain.

## Status
Active. This crate removes direct `app-evm -> app-mem` coupling and centralizes mixed transaction-kind detection.

