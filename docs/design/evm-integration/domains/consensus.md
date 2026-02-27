# Domain: Consensus

## Definition

The consensus domain is responsible for block ordering, finalization, and Byzantine fault detection. It defines the abstract contract that any application must satisfy to participate in consensus.

## Derived crates

| Crate | Role | Status |
|---|---|---|
| `consensus` | Abstract traits: `Block`, `ConsensusApp`, `ConsensusEngine`, `ConsensusEvent`, `EventSink` | Grounded |
| `consensus-simplex` | Simplex BFT adapter: `AppAdapter`, `Mailbox`, `MailboxActor`, `CommonwareEngine` | Grounded |

## Key public contracts

### Block trait
**Grounded**: `crates/consensus/src/block.rs::Block`
```rust
pub trait Block: Send + Sync + 'static {
    type Id: Copy + Eq + Hash + Debug + Send + Sync + 'static;
    fn id(&self) -> Self::Id;
    fn parent_id(&self) -> Self::Id;
    fn height(&self) -> u64;
}
```

### ConsensusApp trait
**Grounded**: `crates/consensus/src/app.rs::ConsensusApp`
```rust
pub trait ConsensusApp: Send + Sync + 'static {
    type Block: Block;
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;
    fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Option<Self::Block>> + Send;
    fn verify(&self, parent: &Self::Block, block: &Self::Block) -> impl Future<Output = Result<(), ConsensusError>> + Send;
}
```

### ConsensusEvent
**Grounded**: `crates/consensus/src/event.rs::ConsensusEvent`
- `Finalized { block, height, proof }`
- `PreFinalized { block, height }`
- `Fault { offender, evidence }`

### EventSink trait
**Grounded**: `crates/consensus/src/event.rs::EventSink`
```rust
pub trait EventSink: Send + Sync + 'static {
    type Block: Block;
    fn handle(&self, event: ConsensusEvent<Self::Block>) -> impl Future<Output = ()> + Send;
}
```

## Core workflows

### Block lifecycle (grounded)
1. Consensus engine calls `app.propose(parent, height)` to get a candidate block
2. Other validators receive the block and call `app.verify(parent, block)`
3. On consensus agreement, `EventSink::handle(Finalized{..})` is called
4. Evidence: `crates/consensus-simplex/src/lib.rs::AppAdapter` bridges these calls

## Integration impact for EVM design

[PROPOSED] The `app` crate's `Application` trait must be compatible with `ConsensusApp`. Options:
1. `Application` extends `ConsensusApp` (supertrait)
2. `Application` is separate, with a blanket impl or adapter from `Application` → `ConsensusApp`
3. `ConsensusApp` gains an associated execution result type

**Recommendation**: Option 2 — keep `ConsensusApp` unchanged, define `Application` in `app` crate with richer block types, and provide an adapter that implements `ConsensusApp` by delegating to `Application`. This preserves backwards compatibility.

## Open questions / TODOs

- **Resolved**: `ConsensusApp::Block` does NOT carry execution results. The `EvmBlock` type (proposed in `app`) carries EVM header fields (state_root, receipts_root, gas_used) as part of the block identity, but these are consensus-visible block fields — not separate execution results. The `ExecutionResult` type lives outside the consensus path and is discarded at the `ApplicationAdapter` boundary. See architecture/consensus-app-bridge.md.
