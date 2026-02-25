# Simplex Adapter Bridge

The Simplex Adapter Bridge acts as a bridge between the vendor consensus stack (commonware-consensus) and the internal consensus-core traits. It provides a shim that translates vendor-specific stream ancestry and contexts into the Whirlpool ConsensusApp and EventSink interfaces.

## Purpose

The adapter layer ensures that the core consensus logic remains decoupled from the specific orchestration details of the Commonware Simplex implementation. It handles the transformation of marshaled vendor data into typed blocks and provides the necessary trait implementations for the vendor stack to drive the internal application.

## Public Types

### CommonwareBlock
```rust
pub trait CommonwareBlock: CoreBlock + VendorBlock + Clone {}
impl<T> CommonwareBlock for T where T: CoreBlock + VendorBlock + Clone {}
```
A composite trait that combines the internal `CoreBlock` with the vendor's `Block` (aliased as `VendorBlock`) and `Clone`. A blanket implementation is provided for any type satisfying these three bounds.

### CommonwareConfig
```rust
pub struct CommonwareConfig {
    pub namespace: String,
    pub leader_timeout: Duration,
    pub notarization_timeout: Duration,
    pub nullify_retry: Duration,
    pub activity_timeout: u64,
    pub skip_timeout: u64,
    pub mailbox_size: usize,
    pub replay_buffer: NonZeroUsize,
    pub write_buffer: NonZeroUsize,
    pub epoch: u64,
    pub fetch_timeout: Duration,
    pub fetch_concurrent: usize,
}
```
Configuration parameters for the Simplex consensus engine.

### AppAdapter
```rust
pub struct AppAdapter<A, S, B, Sig> {
    app: Arc<A>,
    sink: Arc<S>,
    _phantom: PhantomData<(B, Sig)>,
}

pub fn new(app: Arc<A>, sink: Arc<S>) -> Self
where
    A: ConsensusApp<Block = B>,
    S: EventSink<Block = B>,
    B: CommonwareBlock,
    Sig: Scheme,
```
The primary shim implementing vendor traits for the internal application.

### CommonwareEngine
```rust
pub struct CommonwareEngine {
    starter: Box<dyn FnOnce(Arc<AtomicU64>, Arc<AtomicBool>) -> Result<
        (Box<dyn FnOnce() + Send>, JoinHandle<Result<(), ConsensusError>>),
        ConsensusError,
    > + Send>,
}

pub fn new<F>(starter: F) -> Self
where F: FnOnce(Arc<AtomicU64>, Arc<AtomicBool>) -> Result<
    (Box<dyn FnOnce() + Send>, JoinHandle<Result<(), ConsensusError>>),
    ConsensusError,
> + Send + 'static;
```
An implementation of `ConsensusEngine` that delegates the heavy lifting of vendor stack orchestration to a caller-provided closure.

## Adapter Data Flow

| Vendor call | Adapter action | Consensus-core call |
|---|---|---|
| genesis() | Direct delegation | ConsensusApp::genesis() |
| propose(runtime, ctx, ancestry) | Read parent from ancestry | ConsensusApp::propose(&parent, height) |
| verify(runtime, ctx, ancestry) | Read block+parent from ancestry | ConsensusApp::verify(&parent, &block) |
| report(Update::Block) | Forward finalized event | EventSink::handle(Finalized) + ack.acknowledge() |
| report(Update::Tip) | Log only | — |

## Vendor Trait Mapping

The `AppAdapter` implements the following traits from the `commonware-consensus` crate:

*   **Application<E>**: genesis(), propose()
*   **VerifyingApplication<E>**: verify()
*   **Reporter**: report() (Activity = Update<B>)

The bounds for the runtime environment `E` include `Rng`, `Spawner`, `Metrics`, and `Clock` from the `commonware-runtime` crate.

## Design Decisions

*   **Adapter Pattern**: The layer is a thin shim that converts marshaled ancestry streams and vendor contexts into clean `ConsensusApp` calls.
*   **Starter Closure IoC**: `CommonwareEngine` uses Inversion of Control, delegating the complex vendor stack setup to the caller via a closure.
*   **Atomic Status Reporting**: Shared `AtomicU64` and `AtomicBool` are used for cross-thread status tracking without the overhead of locking.
*   **Empty Proof**: The adapter currently sets an empty vector for proofs in `Finalized` events as a placeholder.
*   **Ancestry Ordering**: During proposals, the adapter expects the parent at the head of the stream. During verification, it expects the block followed by the parent.

## File Locations

*   crates/consensus-simplex/src/lib.rs
*   crates/consensus-simplex/src/types.rs
*   crates/consensus-simplex/src/config.rs
*   crates/consensus-simplex/src/adapter.rs
*   crates/consensus-simplex/src/engine.rs
*   crates/consensus-simplex/src/tests.rs
