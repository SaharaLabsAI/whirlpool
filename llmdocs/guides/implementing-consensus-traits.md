# Implementing Consensus Traits

Use canonical trait modules when implementing new consensus integrations.

## Canonical Imports
- `use consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine};`
- `use consensus::{ConsensusError, ConsensusEvent, RunningEngine};`

## Step 1: Implement `Block`
Define `id`, `parent_id`, and `height`.

## Step 2: Implement `ConsensusApp`
Implement async `genesis`, `propose`, `verify` against your `Block` type.

## Step 3: Implement `EventSink`
Handle `ConsensusEvent` values emitted by the engine.

## Step 4: Implement `ConsensusEngine`
Return `RunningEngine` from `start()` and wire shutdown/status.

## Adapter Integration Note
If you target simplex, your block should satisfy `consensus_simplex::traits::CommonwareBlock` in addition to `consensus::traits::Block`.

## Import Migration Rule
Prefer `consensus::traits::*` paths. Do not rely on crate-root trait re-exports.
