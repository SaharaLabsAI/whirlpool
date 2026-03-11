# Task 08 Evidence: Conversion Layer

## Changes Made

### `crates/rpc-eth/src/convert.rs` (NEW)
- `decode_transaction(bytes: &[u8]) → Result<TransactionSigned, Eip2718Error>` — decodes EIP-2718 envelope
- `evmblock_to_header(block: &EvmBlock) → Header` — maps EvmBlock fields to reth Header
- `evmblock_to_block(block: &EvmBlock) → Result<Block, Eip2718Error>` — decodes txs + builds Block

### `crates/rpc-eth/tests/convert_tests.rs` (NEW)
- `decode_transaction_roundtrips_valid_eip1559_bytes` — encode→decode roundtrip
- `decode_transaction_rejects_malformed_bytes` — error on garbage input
- `evmblock_to_header_maps_fields` — verifies all field mappings
- `evmblock_to_block_decodes_transactions` — full block with real txs
- `evmblock_to_block_supports_empty_transactions` — empty tx list

### `crates/rpc-eth/src/lib.rs`
- Added `pub mod convert;`

## Verification

- `nix develop --command cargo build -p rpc-eth` — ✅ passes
- `nix develop --command cargo test -p rpc-eth` — ✅ 34/34 tests pass (17 eth_handler + 5 convert + 5 network + 3 pool + 4 provider)
