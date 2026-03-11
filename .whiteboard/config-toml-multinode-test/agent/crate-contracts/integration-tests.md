# Crate Contract: integration-tests

## Changes Summary
Add multi-node consensus test that verifies P2P connectivity via block height growth.

## New Dependencies
- `whirlpool-node` — node startup API (`start_node`, `NodeConfig`)
- `commonware-cryptography` — ed25519 key generation from seed

## New Test: `tests/multinode_consensus.rs`

### `test_four_node_consensus()`
```rust
#[tokio::test]
async fn test_four_node_consensus() {
    // 1. Generate 4 deterministic signers (seeds 0-3)
    // 2. Collect all 4 public keys as validator set
    // 3. Start 4 nodes with:
    //    - unique seeds, tempdirs, port 0
    //    - shared validator set
    //    - 1s block interval
    // 4. Wire bootstrap peers after port discovery
    // 5. Verification loop (timeout 60s):
    //    a. Poll eth_blockNumber on each node's RPC
    //    b. Log each node's height: "Node {seed}: height={h}"
    //    c. Assert all 4 nodes reach height > 0
    //    d. Assert all 4 heights within ±1 of each other
    // 6. Cleanup: abort all node tasks
}
```

### Peer Connectivity Verification Strategy
- **Primary**: Block height growth across all 4 nodes proves P2P connectivity.
  With 4 validators, BFT requires 2f+1=3 votes — if all 4 produce blocks at the same height, all must be exchanging messages.
- **Secondary**: Node startup adds INFO-level tracing for peer connection events.
  The test uses `tracing_subscriber` to capture logs and verify peer connection events.
- **Tertiary**: Per-node height logging during the polling loop shows each node's progress independently.

## Behavioral Contracts
- BC-1: Test uses only localhost networking, no external dependencies
- BC-2: Test is deterministic (fixed seeds, no randomness)
- BC-3: Test completes within 60s or fails explicitly with timeout error
- BC-4: Each node uses isolated tempdir, cleaned on test completion
- BC-5: All 4 nodes must show block height growth (not just 1)
- BC-6: All 4 nodes' heights must be within ±1 of each other (sync proof)
- BC-7: Tracing logs capture peer connection events for each node
