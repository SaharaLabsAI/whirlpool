# Block Lifecycle & Data Flow

This document maps how blocks move through the three layers of the Whirlpool system. It tracks the flow from the underlying BFT engine up to the application state.

## Three-Layer Architecture

The system splits responsibilities across three distinct layers. This separation ensures the core consensus logic remains independent of the specific BFT implementation.

```
┌─────────────────────────────────────────┐
│  Layer 3: whirlpool-node (binary)       │
│  EmptyBlock, EmptyBlockApp,             │
│  FinalizationSink, Mailbox, wire.rs     │
├─────────────────────────────────────────┤
│  Layer 2: consensus-simplex (adapter)   │
│  AppAdapter, CommonwareEngine,          │
│  CommonwareBlock, CommonwareConfig      │
├─────────────────────────────────────────┤
│  Layer 1: consensus (traits)            │
│  Block, ConsensusApp, EventSink,        │
│  ConsensusEngine, RunningEngine         │
├─────────────────────────────────────────┤
│  Vendor: commonware (git submodule)     │
│  Simplex BFT, P2P, Runtime, Codec,     │
│  Cryptography, Storage                  │
└─────────────────────────────────────────┘
```

## Data Flow Phases

### 1. Proposal Phase
The lifecycle starts when the node becomes the leader for a specific slot.

*   **Trigger**: The Simplex engine identifies the node as the leader.
*   **Marshaled Call**: It calls `Marshaled.propose`, providing an `AncestorStream` containing the parent block.
*   **Adapter Action**: `AppAdapter` extracts the parent block and calculates the next height.
*   **Application Call**: It invokes `ConsensusApp::propose(parent, height)`.
*   **Implementation**: `EmptyBlockApp` creates a new `EmptyBlock`.
*   **Broadcast**: The engine sends the block to the network through the buffer layer.

### 2. Verification Phase
Nodes receive proposals from the network and must validate them before voting.

*   **Trigger**: An incoming proposal arrives via the P2P layer.
*   **Marshaled Call**: The engine calls `Marshaled.verify` with the block and its parent.
*   **Adapter Action**: `AppAdapter` forwards these to the application.
*   **Application Call**: It invokes `ConsensusApp::verify(parent, block)`.
*   **Validation Rules**: `EmptyBlockApp` applies five strict checks:
    1.  Height must be parent height plus one.
    2.  The block's parent ID must match the actual parent ID.
    3.  Non-genesis blocks cannot reference themselves as parents.
    4.  Genesis blocks must have a zeroed out parent ID.
    5.  Implicit validity for the genesis block itself.
*   **Result**: If rules pass, Simplex votes to Notarize. If they fail, it votes to Nullify.

### 3. Finalization Phase
A block becomes final once it collects enough notarization votes.

*   **Trigger**: The engine collects 2f+1 notarization votes.
*   **Event**: Simplex emits an `Update::Block(block, ack)`.
*   **Adapter Action**: `AppAdapter` maps this to a `ConsensusEvent::Finalized` and calls the `EventSink`.
*   **Sink Handling**: `FinalizationSink` receives the event and updates the tracked height.
*   **Acknowledgment**: The adapter calls `ack.acknowledge()` to confirm processing.

## Cross-Crate Type Mappings

This table shows how types relate across the different crates.

| Layer 1 (consensus) | Layer 2 (consensus-simplex) | Layer 3 (whirlpool-node) |
|---|---|---|
| `Block` trait | `CommonwareBlock` | `EmptyBlock` |
| `ConsensusApp` trait | `AppAdapter` | `EmptyBlockApp` |
| `EventSink` trait | `AppAdapter` | `FinalizationSink` |
| `ConsensusEngine` trait | `CommonwareEngine` | — |

## Event Propagation Chain

Events travel from the vendor code through the adapter to the application sink.

1.  **Vendor**: `Update::Block(block, ack)` or `Update::Tip(height, _)`
2.  **Adapter**: `AppAdapter.report` receives the update.
3.  **Mapping**: `Update::Block` becomes `ConsensusEvent::Finalized { block, height, proof: vec![] }`.
4.  **Sink**: `EventSink::handle` processes the mapped event.

## Observability

The system uses atomics to expose state across thread boundaries without locking.

*   **FinalizationSink**: Holds an `Arc<AtomicU64>` to store the latest finalized height.
*   **RunningEngine**: The `status()` method reads from this shared atomic value.
*   **ConsensusStatus**: Returns the current height and engine health to callers.
