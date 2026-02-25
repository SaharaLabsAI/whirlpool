# Whirlpool Node Architecture: LLM Retrieval Map

The Whirlpool node binary implements the core execution engine for the Sahara Chain, providing a minimal consensus node implementation.

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

### sink.rs
- pub struct FinalizationSink { height: Arc<AtomicU64> }
- pub fn FinalizationSink::new(height: Arc<AtomicU64>) -> Self;
- pub fn FinalizationSink::current_height(&self) -> u64;

### mailbox.rs
- pub enum Message { Genesis { epoch: Epoch, response: oneshot::Sender<Digest> }, Propose { response: oneshot::Sender<Digest> }, Verify { digest: Digest, response: oneshot::Sender<bool> } }
- pub struct Mailbox { sender: mpsc::Sender<Message> }
- pub struct MailboxActor { receiver: mpsc::Receiver<Message>, height: Arc<AtomicU64> }
- pub fn MailboxActor::new(receiver, height) -> Self;
- pub async fn MailboxActor::run(mut self);

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

## CFG Gating

- mailbox.rs: #[cfg(any(test, feature = "never_enable_this"))]
- sink.rs: #[cfg(any(test, feature = "never_enable_this"))]

These modules only compile during tests or when the specific feature is enabled.

## Stub Status

### wire.rs (STUB)
- Current: Spawns std::thread polling running flag every 100ms.
- Missing: No wiring for simplex engine, P2P, mailbox actor, or app adapter.
- Signature: pub fn create_starter() -> impl FnOnce(Arc<AtomicU64>, Arc<AtomicBool>) -> Result<...> + Send + 'static

### main.rs (STUB)
- Current: Only prints "Sahara Chain Binary".
- Missing: No tracing setup, engine creation, Ctrl-C handler, or shutdown logic.

## Test Statistics

| Module | Test Count |
|---|---|
| block.rs | 8 |
| app.rs | 11 |
| sink.rs | 6 |
| mailbox.rs | 7 |
| **Total** | **32** |
