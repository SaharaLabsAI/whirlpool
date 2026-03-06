# Alignment Digest

## Approved Scope
Persistent block storage (full blocks: headers, bodies, transactions, receipts) via MDBX + history block queries via eth_getBlock* RPC endpoints.

## Approach (3 streams, single intent)
1. **BlockStorage trait** in `state` + impl in `state-reth` (MDBX tables: Headers, BlockBodyIndices, Transactions, Receipts, HeaderNumbers)
2. **Finalization persistence hook**: persist blocks on finalization, flow receipts from execution
3. **RPC endpoints**: eth_getBlockByHash, eth_getBlockByNumber in rpc-eth

## Key Decisions
- Use existing EvmBlock→reth Header conversion (build_header_from_evm_block)
- Use existing decode_transactions() for raw→typed tx conversion
- Persistence at application layer (app-evm), not consensus-simplex (due to generic Block constraint)
- Batch MDBX writes per block for performance
- Extend finalization event to carry receipts

## Crate Impact
state (trait), state-reth (impl), app/app-evm (receipt flow, conversions), consensus-simplex (minor: event extension), rpc-eth (block endpoints), whirlpool-node (wiring)

## Risks
- 0 BLOCKERS, 2 MEDIUM (receipt flow gap, generic type constraint), 2 LOW — all mitigated
- Type encoding mismatch RESOLVED (conversion functions exist)

## User Decision
- Approved: 2026-03-06
