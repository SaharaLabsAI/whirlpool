# Crate Contract: app

## Role
Defines core EVM application types (`EvmBlock`, `ExecutionResult`, `BlockId`) and consensus adapter types. Shared foundation for all EVM-related crates.

## Changes Required
- **Minor**: Re-export `Receipt` type from `alloy-consensus` for BlockStorage trait signature consistency
- No structural changes to `EvmBlock` or other existing types

## Public API (After Changes)

### Existing (unchanged)
```rust
// app/src/types.rs
pub struct EvmBlock {
    pub height: u64,
    pub parent_id: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub timestamp: u64,
    pub transactions: Vec<Vec<u8>>,  // raw EIP-2718 encoded tx bytes
}

pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}
```

### New
```rust
// app/src/lib.rs (or types.rs)
pub use alloy_consensus::Receipt;  // re-export for BlockStorage trait consumers
```

## New Dependencies
- `alloy-consensus = "1.4.3"` — for Receipt type re-export

## Files Changed
- `src/lib.rs` — add `alloy-consensus` dependency and `Receipt` re-export

## Tests
No new tests required — this is a type re-export only.

## Integration Points
- **state**: `BlockStorage` trait references `Receipt` type → consumers can import from `app` crate
- **app-evm**: Already depends on `app`, gains access to `Receipt` re-export
- **rpc-eth**: Uses `Receipt` for response conversion

## Grounding Evidence
- `app/src/types.rs` (lines 21-30): EvmBlock definition
- STRATEGY.md: "app/src/types.rs: No structural changes to EvmBlock"
- CRATES.md: "app — Minor — Re-export Receipt type from alloy-consensus"
