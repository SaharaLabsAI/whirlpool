# Empty Block Cadence (5s)

## 1. Contract

The chain emits exactly one canonical block per height, where each block is an **empty block** and target cadence is **5 seconds per height**.

- Payload is always empty.
- Height increments by 1 from parent.
- Parent linkage must be exact.
- Block timestamp is deterministic from height.

## 2. Block model

```rust
pub struct EmptyBlock {
    pub id: [u8; 32],
    pub parent_id: [u8; 32],
    pub height: u64,
    pub timestamp_unix_secs: u64,
    pub payload: Vec<u8>, // always empty in v0
}
```

`EmptyBlock` must satisfy both trait surfaces:

- `consensus_core::Block`
- `consensus_commonware::CommonwareBlock` (via required Commonware traits)

## 3. Slot schedule

Let:

- `GENESIS_TIME = genesis_time_unix_secs`
- `BLOCK_INTERVAL = 5`
- `slot_time(height) = GENESIS_TIME + (height * BLOCK_INTERVAL)`

For each proposed block at `height`:

1. Compute `target = slot_time(height)`.
2. If local wall clock is before `target`, wait until `target`.
3. Produce empty block with:
   - `block.height = height`
   - `block.timestamp_unix_secs = target`
   - `block.payload = []`

This keeps timestamps deterministic while pacing production in real time.

## 4. Propose and verify rules

### Propose (`ConsensusApp::propose`)

- Input: `parent`, `height`.
- Preconditions:
  - `height == parent.height() + 1`
- Output:
  - `Some(empty_block)` on success.
  - `None` is not used in v0 (binary always attempts to fill the slot).

### Verify (`ConsensusApp::verify`)

Reject unless all checks pass:

1. `block.parent_id == parent.id()`
2. `block.height == parent.height() + 1`
3. `block.payload.is_empty()`
4. `block.timestamp_unix_secs == parent.timestamp_unix_secs + 5`
5. `block.id` matches canonical hash of block fields

Return `ConsensusError::InvalidBlock(...)` for failed checks.

## 5. Finalization behavior

On `ConsensusEvent::Finalized { block, height, .. }`:

- Assert `height` is strictly increasing by 1.
- Persist finalized `(height, block.id, block.timestamp_unix_secs)`.
- Publish metrics:
  - `chain_finalized_height`
  - `chain_finality_lag_seconds = now - block.timestamp_unix_secs`

## 6. Acceptance criteria

1. Single node run finalizes empty blocks at ~5s cadence.
2. Finalized block timestamps increase by exactly 5 seconds.
3. No finalized block contains payload bytes.
4. Restart from persisted head continues with correct next height and timestamp progression.
