# Consensus Trait Layer Architecture

The consensus crate provides the stable trait boundary used by adapter and node crates.

## Canonical Trait Module
- `crates/consensus/traits/src/traits.rs` is the canonical import surface.
- Canonical paths:
  - `consensus::traits::Block`
  - `consensus::traits::ConsensusApp`
  - `consensus::traits::EventSink`
  - `consensus::traits::ConsensusEngine`
- Engine/runtime types remain in crate root exports:
  - `consensus::RunningEngine`
  - `consensus::ConsensusStatus`
  - `consensus::ConsensusError`
  - `consensus::ConsensusEvent`

## Public Signatures

### Block (`consensus::traits::Block`)
`fn id(&self) -> Self::Id; fn parent_id(&self) -> Self::Id; fn height(&self) -> u64`

### ConsensusApp (`consensus::traits::ConsensusApp`)
`fn genesis(&self) -> impl Future<Output = Self::Block> + Send`
`fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Option<Self::Block>> + Send`
`fn verify(&self, parent: &Self::Block, block: &Self::Block) -> impl Future<Output = Result<(), ConsensusError>> + Send`

### EventSink (`consensus::traits::EventSink`)
`fn handle(&self, event: ConsensusEvent<Self::Block>) -> impl Future<Output = ()> + Send`

### ConsensusEngine (`consensus::traits::ConsensusEngine`)
`fn start(self) -> Result<RunningEngine, ConsensusError>`

## Design Notes
- Interface/implementation split is explicit: trait boundary is centralized in `traits.rs`.
- Downstream crates should prefer `consensus::traits::*` imports.

## File Locations
- `crates/consensus/traits/src/traits.rs`
- `crates/consensus/traits/src/block.rs`
- `crates/consensus/traits/src/app.rs`
- `crates/consensus/traits/src/event.rs`
- `crates/consensus/traits/src/engine.rs`
