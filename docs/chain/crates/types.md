# `types`

**Purpose**: shared data model + cryptographic primitives used everywhere.

Owns: IDs/hashes/roots, block/tx structs, signatures, basic validation helpers, (de)serialization.

Consensus-adjacent (minimal): `Height`, `Round/View`, `ValidatorId`, `Vote`, `Certificate`, `NotarizedBlock`, `FinalizedBlock`, and `Quorum`/threshold params.

## Block (shape)

```rust
// NOTE: high-level shapes, not a spec.

pub type BlockId = Hash;
pub type ValidatorId = Bytes; // e.g. public key or address
pub type Height = u64;

pub struct Block {
  pub header: BlockHeader,
  pub body: BlockBody,
}

pub struct BlockHeader {
  pub parent: BlockId,
  pub height: Height,
  pub timestamp: u64,
  pub proposer: ValidatorId,
  pub tx_root: Hash,
  pub state_root: Hash,
}

pub struct BlockBody {
  // Blocks contain *signed* transactions only.
  pub txs: Vec<SignedTransaction>,
}
```

## Transaction (shape)

We support multiple transaction families. EVM is one variant; others can be added later.

Blocks only include **signed** transactions. Unsigned transactions exist as a client-side / builder-side
representation (e.g. wallet construction) and are not accepted by RPC.

```rust
// NOTE: high-level shapes, not a spec.

// Unsigned transaction (pre-signing).
pub enum Transaction {
  Evm(EvmUnsignedTx),

  // Future extension point for non-EVM tx families.
  // `type_id` is a stable discriminant; `payload` is family-defined bytes.
  Other { type_id: u32, payload: Bytes },
}

// Signed transaction (ready for gossip, mempool, execution, and inclusion in blocks).
pub enum SignedTransaction {
  Evm(EvmSignedTx),

  // Future extension point for non-EVM tx families.
  // `payload` is family-defined signed bytes (including any signature material).
  Other { type_id: u32, payload: Bytes },
}

// EVM typed transaction envelopes (reth/alloy).
pub type EvmUnsignedTx = reth_ethereum_primitives::Transaction;
pub type EvmSignedTx = reth_ethereum_primitives::TransactionSigned;
```

## Proofs (separate from the block)

Blocks are sealed; consensus proofs/certificates accumulate during voting and are stored separately.

```rust
// Opaque commitment to a quorum proof (QC / aggregated signatures / etc.).
pub type Certificate = Bytes;

pub struct NotarizedBlock {
  pub certificate: Certificate,
  pub block: Block,
}

pub struct FinalizedBlock {
  pub certificate: Certificate,
  pub block: Block,
}
```

Inputs/outputs: pure types + helper functions (no IO).

Depends on: minimal crypto + codec crates only.

Not in scope: networking, consensus rules, storage engines.
