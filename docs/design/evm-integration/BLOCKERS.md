# Blockers

## Summary
- Total: 6 blockers (2 scope-expansion, 1 information-gap, 1 decision-gap, 2 new)
- Resolved this round: 4 (B-001 pending, B-003/B-004 new)
## Active blockers

### [B-001] ChainSpec selection
- **Type**: `decision-gap`
- **Severity**: `blocking` (cannot implement without resolution)
- **Affected crates**: `app-evm`
- **Description**: `WhirlpoolEvmConfig::new(chain_spec: Arc<ChainSpec>)` requires a `ChainSpec` value. No chain ID, genesis configuration, or hardfork schedule exists anywhere in the Whirlpool workspace (`crates/`). The `ChainSpec` determines chain ID, genesis state, and which EVM hardforks are active at which block heights. Reth provides `ChainSpecBuilder` for construction.
- **Suggested resolution**: Product/team decision required — choose a chain ID for Sahara, decide on genesis allocations, and specify hardfork activation schedule (at minimum Shanghai + Cancun). Once decided, construct via `ChainSpecBuilder::default().chain(SAHARA_CHAIN_ID).genesis(genesis).shanghai_activated().cancun_activated().build()`. Could live in `app-evm` as a constant or be loaded from a config file in `whirlpool-node`.

### ~~[B-002] State database implementation~~ → RESOLVED
- **Type**: `scope-expansion` → **resolved (round 2)**
- **Resolution**: New `state` crate provides `InMemoryStateDb` implementing `revm::Database + Clone`. Provides `commit(&BundleState)` for state commitment and `state_root() -> B256` for block headers. `EvmApplication<DB>` is instantiated with `DB = InMemoryStateDb`, wrapped in `Arc<RwLock<InMemoryStateDb>>` for interior mutability (`&self` trait compatibility). Speculative execution uses clone-based snapshots; canonical state is committed only on finalization. See `state/README.md`, `wiring/state-storage.md`, `domains/state-storage.md`.
- **Affected docs**: `state/README.md` (NEW), `app-evm/README.md`, `wiring/state-storage.md` (NEW), `domains/state-storage.md` (NEW), `architecture/block-proposal.md`, `architecture/block-verification.md`, `architecture/node-startup.md`

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

### [B-003] Merkle Patricia Trie for state root computation
- **Type**: `scope-expansion`
- **Severity**: `blocking` (for production; MVP uses flat keccak256 hash)
- **Affected crates**: `state`
- **Description**: `InMemoryStateDb::state_root()` currently uses a flat keccak256 hash over sorted accounts and storage. This is deterministic and sufficient for consensus agreement, but does NOT produce Ethereum-compatible state roots. Production requires a Merkle Patricia Trie (or Verkle trie) for: state proofs (`eth_getProof`), light client verification, and Ethereum JSON-RPC compatibility.
- **Suggested resolution**: Integrate an MPT library (e.g., `alloy-trie` or reth's `reth-trie`) into the `state` crate. The `state_root()` method signature is unchanged — only the internal algorithm swaps from flat hash to trie computation.

### [B-004] State persistence (in-memory only)
- **Type**: `scope-expansion`
- **Severity**: `blocking` (for production; MVP is in-memory only)
- **Affected crates**: `state`
- **Description**: `InMemoryStateDb` stores all state in HashMaps. State is lost on process restart. Production requires a persistent backend (RocksDB, MDBX, or similar) to survive restarts and support nodes joining the network after genesis.
- **Suggested resolution**: Implement a persistent `Database + Clone` backend (e.g., `RocksDbStateDb`) that can be swapped in via `EvmApplication<DB>` generic. The in-memory implementation remains useful for testing and short-lived nodes.
