# Block Lifecycle Walkthrough

This guide traces a block from its initial proposal through network verification to its final commitment in the node state. It follows the execution flow across the three system layers.

## Proposal Flow

When the node is selected as the leader for a slot, it must produce a new block.

### Execution Path
1.  **Vendor (Simplex)**: Identifies the leader and calls `Marshaled.propose`.
2.  **Adapter (`consensus-simplex`)**:
    *   `AppAdapter.propose(runtime, ctx, ancestry)` is called.
    *   It retrieves the `parent` block from the `ancestry` stream.
    *   It calculates the new height using `Heightable::height(&parent).next().get()`.
3.  **Interface (`consensus`)**: Calls the `ConsensusApp::propose(&parent, height)` trait method.
4.  **Application (`whirlpool-node`)**:
    *   `EmptyBlockApp::propose` executes.
    *   It constructs an `EmptyBlock` with the provided parent ID and height.
5.  **Return Path**: The new block returns through the adapter to the Simplex engine for network broadcast.

### Diagram
```
Simplex (Vendor)
  │
  └─> AppAdapter.propose(runtime, ctx, ancestry)
        │
        └─> ConsensusApp::propose(parent, height)
              │
              └─> EmptyBlockApp::propose(...) -> EmptyBlock
```

## Verification Flow

When the node receives a block proposed by another leader, it must verify the block's validity.

### Execution Path
1.  **Vendor (Simplex)**: Receives a block and calls `Marshaled.verify`.
2.  **Adapter (`consensus-simplex`)**:
    *   `AppAdapter.verify(runtime, ctx, ancestry)` is called.
    *   It extracts both the proposed `block` and its `parent` from the `ancestry` stream.
3.  **Interface (`consensus`)**: Calls `ConsensusApp::verify(&parent, &block)`.
4.  **Application (`whirlpool-node`)**:
    *   `EmptyBlockApp::verify` checks the block against five internal rules.
    *   **Rule 1**: The block height must equal the parent height plus one.
    *   **Rule 2**: The block's stored parent ID must match the actual ID of the parent block.
    *   **Rule 3**: If the block is not the genesis block, its ID cannot be the same as its parent ID.
    *   **Rule 4**: If the block is the genesis block (height 0), its parent ID must be a zeroed 32-byte array.
    *   **Rule 5**: The genesis block is always considered valid if it meets the basic format.
5.  **Return Path**: A boolean result returns to the adapter. Simplex then issues a `Notarize` vote if valid, or a `Nullify` vote if invalid.

### Diagram
```
Simplex (Vendor)
  │
  └─> AppAdapter.verify(runtime, ctx, ancestry)
        │
        └─> ConsensusApp::verify(parent, block)
              │
              └─> EmptyBlockApp::verify(...) -> bool
```

## Finalization Flow

Finalization occurs when a block gains enough notarization votes from the network.

### Execution Path
1.  **Vendor (Simplex)**: Collects 2f+1 votes and commits the block. It emits an `Update::Block(block, ack)`.
2.  **Adapter (`consensus-simplex`)**:
    *   `AppAdapter.report(update)` is called.
    *   It matches the update type. `Update::Tip` is logged but ignored for state changes.
    *   `Update::Block` is converted into a `ConsensusEvent::Finalized` instance.
3.  **Interface (`consensus`)**: Calls `EventSink::handle(ConsensusEvent)`.
4.  **Application (`whirlpool-node`)**:
    *   `FinalizationSink::handle` receives the `Finalized` event.
    *   It extracts the block height and updates its internal `Arc<AtomicU64>` for observability.
5.  **Completion**: The adapter calls `ack.acknowledge()` to notify the vendor that the block was successfully processed.

### Diagram
```
Simplex (Vendor)
  │
  └─> AppAdapter.report(Update::Block(block, ack))
        │
        └─> EventSink::handle(ConsensusEvent::Finalized)
              │
              └─> FinalizationSink::handle(...)
                    │
                    └─> ack.acknowledge()
```
