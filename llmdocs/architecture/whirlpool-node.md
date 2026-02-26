# Whirlpool Node Architecture: LLM Retrieval Map

The Whirlpool node binary provides a minimal consensus node implementation, delegating consensus wiring to the consensus-simplex library. The node focuses purely on business logic: EmptyBlock definition and EmptyBlockApp verification rules.

## Type Signatures

### block.rs
- pub type BlockId = [u8; 32];
- pub struct EmptyBlock { height: u64, parent_id: BlockId }
- pub fn EmptyBlock::genesis() -> Self;
- pub fn EmptyBlock::new(height: u64, parent_id: BlockId) -> Self;
- fn compute_id(&self) -> BlockId

### app.rs
- pub struct EmptyBlockApp;
- pub fn EmptyBlockApp::new() -> Self;
- async fn EmptyBlockApp::genesis(&self) -> EmptyBlock;
- async fn EmptyBlockApp::propose(&self, parent: &EmptyBlock, height: u64) -> Option<EmptyBlock>;
- async fn EmptyBlockApp::verify(&self, parent: &EmptyBlock, block: &EmptyBlock) -> Result<(), ConsensusError>;

### config.rs
- pub const NAMESPACE: &[u8];
- pub const BLOCK_INTERVAL: Duration;
- pub const VALIDATOR_SEED: [u8; 32];
- pub const BIND_ADDR: &str;

## Dual-Trait Conformance: EmptyBlock

| Trait | Details |
|---|---|
| consensus::Block | type Id = BlockId; id(), parent_id(), height() |
| commonware_codec::Write | Writes u64 height + [u8;32] parent_id (40 bytes) |
| commonware_codec::Read | Reads 40 bytes; type Cfg = () |
| commonware_codec::EncodeSize | Returns 40 |
| commonware_cryptography::Digestible | type Digest = sha256::Digest; wraps compute_id() |
| commonware_cryptography::Committable | type Commitment = sha256::Digest; delegates to digest() |
| commonware_consensus::Heightable | Returns Height::new(self.height) |

## EmptyBlockApp Verification Rules

1. Height increment: block.height == parent.height + 1
2. Parent ID match: block.parent_id() == parent.id()
3. No self-reference (non-genesis): block.id() != block.parent_id() unless height == 0
4. Genesis parent zero: height 0 blocks must have parent_id == [0u8; 32]
5. Implicit genesis validity: Covered by rules above

## Architecture Changes (Post-Refactor)

Mailbox, FinalizationSink, and Wire modules have been moved to consensus-simplex. The node now:
1. Defines EmptyBlock and EmptyBlockApp (business logic only)
2. Imports CommonwareEngine from consensus-simplex
3. Constructs engine with app, sink, config in main/tests
4. Sealed engine wiring handles all consensus plumbing
5. Zero consensus infrastructure remains in whirlpool-node

## Test Statistics

| Module | Test Count |
|--------|-----------|
| block.rs | 8 |
| app.rs | 11 |
| **Total** | **19** |
