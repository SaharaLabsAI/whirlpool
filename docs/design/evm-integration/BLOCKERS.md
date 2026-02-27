# Blockers

## Summary
- Total: 4 blockers (2 scope-expansion, 1 information-gap, 1 decision-gap)
- Resolved this round: 2

## Active blockers

### [B-001] ChainSpec selection
- **Type**: `decision-gap`
- **Severity**: `blocking` (cannot implement without resolution)
- **Affected crates**: `app-evm`
- **Description**: `WhirlpoolEvmConfig::new(chain_spec: Arc<ChainSpec>)` requires a `ChainSpec` value. No chain ID, genesis configuration, or hardfork schedule exists anywhere in the Whirlpool workspace (`crates/`). The `ChainSpec` determines chain ID, genesis state, and which EVM hardforks are active at which block heights. Reth provides `ChainSpecBuilder` for construction.
- **Suggested resolution**: Product/team decision required — choose a chain ID for Sahara, decide on genesis allocations, and specify hardfork activation schedule (at minimum Shanghai + Cancun). Once decided, construct via `ChainSpecBuilder::default().chain(SAHARA_CHAIN_ID).genesis(genesis).shanghai_activated().cancun_activated().build()`. Could live in `app-evm` as a constant or be loaded from a config file in `whirlpool-node`.

### [B-002] State database implementation
- **Type**: `scope-expansion`
- **Severity**: `blocking` (cannot implement without resolution)
- **Affected crates**: `app-evm`
- **Description**: `EvmApplication<DB: Database + Clone>` requires a concrete `Database` implementation for EVM state storage. The `Database` trait (from `revm`) provides `basic(Address) -> AccountInfo`, `code_by_hash(B256) -> Bytecode`, `storage(Address, U256) -> U256`, and `block_hash(u64) -> B256`. Reth has `StateProvider`/`StateProviderFactory` abstractions that wrap `Database`, plus various backends (MDBX, in-memory). None of these are in the Whirlpool workspace.
- **Required interface**: The out-of-scope crate (e.g. `state` or `storage`) must provide:
  ```rust
  /// Concrete type implementing revm::Database + Clone.
  /// Must support:
  ///   - Reading account info (balance, nonce, code_hash)
  ///   - Reading storage slots
  ///   - Reading block hashes for BLOCKHASH opcode
  ///   - Committing BundleState diffs after finalization
  pub trait StateDb: revm::Database<Error = StateError> + Clone {
      /// Commit a BundleState (post-execution state diff) to persistent storage.
      fn commit(&mut self, bundle: revm::db::BundleState) -> Result<(), StateError>;
      /// Compute the state root after applying pending changes.
      fn state_root(&self) -> Result<[u8; 32], StateError>;
  }
  ```
  For initial integration, `revm::db::CacheDB<revm::db::EmptyDB>` can serve as an in-memory placeholder (implements `Database + Clone`), though it lacks persistence and state root computation.
- **Suggested resolution**: New design round for a `state` crate, or adopt reth's `StateProviderFactory` pattern. For MVP, use `CacheDB<EmptyDB>` with a stub `state_root()`.

## Resolved blockers (this round)

### [B-R01] EvmBlock serialization (commonware codec traits)
- **Type**: `information-gap` → **resolved**
- **Resolution**: Investigated `EmptyBlock` implementation in `crates/whirlpool-node/src/block.rs`. EvmBlock must implement 7 traits from 3 commonware crates:
  1. `commonware_codec::CodecWrite` — manual binary `fn write(&self, buf: &mut impl BufMut)`
  2. `commonware_codec::EncodeSize` — `fn encode_size(&self) -> usize`
  3. `commonware_codec::CodecRead` — `fn read_cfg(reader: &mut impl Buf, _cfg: &()) -> Result<Self, CodecError>`
  4. `commonware_cryptography::Digestible` — `type Digest = sha256::Digest; fn digest(&self) -> Digest` (SHA-256 of serialized content)
  5. `commonware_cryptography::Committable` — `type Commitment = sha256::Digest; fn commitment(&self)` (delegates to `digest()`)
  6. `commonware_consensus::Heightable` — `fn height(&self) -> Height` (wraps `u64`)
  7. `commonware_consensus::VendorBlock` — `fn parent(&self) -> Self::Commitment`

  Engine bounds: `CommonwareBlock + Digestible<Digest = sha256::Digest> + Send + Sync + 'static`. Pattern is straightforward: serialize EvmBlock fields to bytes (u64 as LE + fixed-size byte arrays + length-prefixed vecs), hash with SHA-256 for identity.
- **Affected docs**: `app/README.md` (blocker note updated), `domains/application.md`

### [B-R02] Transaction source for propose()
- **Type**: `scope-expansion` → **resolved as information-gap**
- **Resolution**: Investigated reth's `ConfigureEvm::builder_for_next_block()` pattern. The block builder receives transactions one-by-one via `execute_transaction(tx)` — it does NOT pull from a pool. The caller (payload builder / proposer) iterates a transaction source and feeds transactions. Reth's `TransactionPool` trait (`vendor/reth/crates/transaction-pool/src/traits.rs`) is a large interface (~50 methods) that is overkill for Whirlpool's needs. The `app-evm` crate can define a minimal trait:
  ```rust
  /// Minimal transaction source for block proposal.
  /// [PROPOSED] — lives in app-evm.
  pub trait TxSource: Send + Sync {
      /// Return pending transactions ordered by priority.
      fn pending(&self) -> Vec<TransactionSigned>;
  }
  ```
  `EvmApplication` takes `TxSource` as a generic or field. For MVP / empty-block mode, a `NoopTxSource` (returns empty vec) suffices — this matches the current `EmptyBlockApp` behavior. Full transaction pool integration is a separate scope-expansion concern for a future `tx-pool` crate.
- **Affected docs**: `app-evm/README.md` (blocker note updated), `architecture/block-proposal.md`
