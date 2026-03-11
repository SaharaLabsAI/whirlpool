# Task 03 Evidence: Provider block and header readers

## Summary

Replaced noop stubs for BlockHashReader, BlockNumReader, HeaderProvider,
BlockReader, and BlockBodyIndicesProvider in `crates/rpc-eth/src/provider.rs`
with real MDBX-backed implementations that delegate to `RethStateDb.inner().tx()`.

## Changes

### Modified files
- **`crates/rpc-eth/src/provider.rs`** — Real impls for 5 provider traits

### Traits implemented (real MDBX reads)
| Trait | Methods |
|-------|---------|
| BlockHashReader | block_hash, canonical_hashes_range |
| BlockNumReader | chain_info, best_block_number, last_block_number, block_number |
| HeaderProvider | header, header_by_number, header_td_by_number, headers_range, sealed_header, sealed_headers_range, sealed_headers_while |
| BlockReader | find_block_by_hash, block, pending_block (noop), block_with_senders, sealed_block_with_senders, block_range, block_with_senders_range, sealed_block_with_senders_range |
| BlockBodyIndicesProvider | block_body_indices, block_body_indices_range |

### Data access pattern
All reads go through `self.state_db.inner().tx()` using reth_db_api::Database trait.
Tables used: CanonicalHeaders, HeaderNumbers, Headers, BlockBodyIndices,
Transactions, HeaderTerminalDifficulties.

## Verification

- `cargo build -p rpc-eth`: **PASS**
- `cargo test -p rpc-eth --lib`: **PASS** (17/17 tests)
- `cargo test -p rpc-eth --test provider_contract`: **PASS** (1/1 test)
- No vendor files modified

## Artifact Coverage
- REQ-2: ✅ Provider reads blocks/headers from storage
- TST-1: ✅ Provider contract test still passes
- TST-6/TST-8: Partial (tests will be added in Task 12)

## Timestamp
2026-03-11T07:15:00Z
