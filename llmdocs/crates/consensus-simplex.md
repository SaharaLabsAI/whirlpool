# consensus-simplex: Simplex BFT Adapter

## Summary
The `consensus-simplex` crate provides an adapter that bridges Whirlpool's abstract consensus traits to the Commonware Simplex BFT implementation.

Location: `crates/consensus-simplex/`

## Key Components

### CommonwareConfig
Holds parameters for the Simplex engine.
- `height: Arc<AtomicU64>`: Caller-owned shared height tracker used for recovery and block production (crates/consensus-simplex/src/config.rs:54).

### AppAdapter
Bridges `ConsensusApp` and `EventSink` to vendor traits.
- Implements `Application`, `VerifyingApplication`, and `Reporter` (crates/consensus-simplex/src/adapter.rs:89,124,153).
- Trait bounds for `Application`, `VerifyingApplication`, and `Reporter` do not require `S: Clone` as the sink is accessed via `Arc` (crates/consensus-simplex/src/adapter.rs:93,128,156).

### CommonwareEngine
The primary entry point for starting the consensus engine.
- Uses the caller-provided `EventSink` passed to `AppAdapter` for finalization events (crates/consensus-simplex/src/engine.rs:188).
- Shared `height` Arc is passed to `MailboxActor` to track the current chain tip (crates/consensus-simplex/src/engine.rs:165).

## Data Flow
1. **Proposal**: `CommonwareEngine` -> `MailboxActor` reads `height` -> `ConsensusApp::propose`.
2. **Finalization**: Vendor engine -> `AppAdapter::report` -> `EventSink::handle(Finalized)`.
3. **Persistence**: `PersistingFinalizationSink` (in `whirlpool-node`) receives event -> stores block -> increments shared `height` Arc.
