# Exploration Digest

## Key Findings (≤500 tokens)

**Block Storage Gap**: Finalized blocks are dropped after consensus processing. Only block number→hash mapping persists (via StateDb/CanonicalHeaders). No full block storage exists.

**MDBX Tables Ready**: init_db() already creates all reth-db tables including Headers, BlockBodyIndices, Transactions, TransactionHashNumbers, Receipts. These are empty but structurally available.

**Type Mismatch [RISK]**: EvmBlock uses commonware_codec binary encoding. reth-db tables expect Compact trait encoding. Cannot directly store EvmBlock in reth Headers table. Need conversion layer or custom storage approach.

**Finalization Hook**: AppAdapter::report(Finalization) is the ideal persistence hook — has the finalized block in hand before forwarding to EventSink. Alternative: new persistence sink alongside FinalizationSink.

**RPC Extension**: rpc-eth has 7 endpoints, zero block endpoints. Framework (jsonrpsee) supports easy addition. Need: block data source in EthRpcContext, methods in EthApiServer trait, EvmBlock→alloy_rpc_types::Block conversion.

**Architecture Decision**: Either (a) extend state-reth with BlockStore trait + impl, or (b) create new block-store crate. Prior pattern: trait in `state`, impl in `state-reth`.

## Risks
1. **Type encoding mismatch** — EvmBlock ↔ reth Header conversion [UNKNOWN complexity]
2. **Transaction decoding** — EvmBlock stores txs as Vec<Vec<u8>>; need to recover typed transactions for RPC responses
3. **Receipt storage** — receipts currently in-memory ReceiptStore; need to reconcile with persistent storage
4. **Finalization performance** — MDBX write on every finalization may impact consensus latency
