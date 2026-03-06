# Shared Context

## Architecture Summary
Whirlpool: 3-layer Rust consensus framework
- Layer 1: `consensus` crate — `Block`, `ConsensusApp`, `EventSink`, `ConsensusEngine` traits
- Layer 2: `consensus-simplex` — Commonware Simplex adapter (MailboxActor, AppAdapter, FinalizationSink)
- Layer 3: Node binaries (`whirlpool-node`)
- EVM path: app → app-evm → state (trait) → state-memory / state-reth (impl)
- Networking: p2p → p2p-commonware

## Current Block Flow
1. **Proposal**: Simplex → AppAdapter.propose → ConsensusApp::propose → returns serialized Block
2. **Verification**: Simplex → AppAdapter.verify → ConsensusApp::verify → 5-rule check
3. **Finalization**: Simplex → AppAdapter.report(Finalized) → retrieves Block from in-memory BlockStore → EventSink::handle(Finalized(block)) → FinalizationSink updates height counter → ack
4. **Post-finalization**: Block is dropped. No persistence. No query path.

## Key Types
- `Block` trait: `id() -> [u8;32]`, `parent_id() -> [u8;32]`, `height() -> u64`
- `EmptyBlock`: height + parent_id (test block, no txs)
- `BlockStore<B>`: `Arc<RwLock<HashMap<Digest, B>>>` (ephemeral, in consensus-simplex)
- `StateDb` trait: 11 methods including `insert_block_hash/get_block_hash` (number→hash only)

## Existing MDBX Patterns (from state-reth)
- reth-db vendor stack: MDBX tables, `DatabaseEnv`, `tx_ref()`, `tx_mut()`
- CanonicalHeaders table already stores block number → hash mapping
- Error taxonomy: 4-tier (Database, State, Internal, Config)
- Pattern: trait in `state` crate, impl in `state-reth` crate

## Unknowns
- [ ] Which reth-db tables exist for block headers/bodies/transactions/receipts
- [ ] Current rpc-eth endpoint implementations (what exists today)
- [ ] How Block types serialize for storage (Alloy RLP? custom?)
- [ ] Whether consensus-simplex Block generic `B` can be stored generically or needs concrete type
