# Task 4: 4-Node Integration Test with P2P + Height Verification

**Complexity**: L
**Covers**: AC-5, AC-6, AC-7, INV-2, INV-3

## Pre-Task Gate
- [ ] Task 3 complete (start_node() exported, NodeHandle available)
- [ ] `nix develop --command cargo build` passes (full workspace)
- [ ] `whirlpool_node::start_node` and `whirlpool_node::NodeConfig` are importable

## What to Do

### Step 1: Add dependencies to integration-tests
In `testing/integration-tests/Cargo.toml`:
```toml
whirlpool-node = { path = "../../crates/whirlpool-node" }
commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Step 2: Write test file (behavior test)
Create `testing/integration-tests/tests/multinode_consensus.rs`:

```rust
use whirlpool_node::{start_node, NodeConfig, NodeHandle};
use commonware_cryptography::{Ed25519, Signer, Scheme};
use std::time::Duration;
use tokio::time::{timeout, sleep};

#[tokio::test]
async fn test_four_node_consensus() {
    // Initialize tracing for peer connection logs
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let num_nodes = 4;
    
    // 1. Generate 4 deterministic signers
    let signers: Vec<Ed25519> = (0..num_nodes)
        .map(|seed| Ed25519::from_seed(seed as u64))
        .collect();
    
    // 2. Collect all public keys as validator set
    let validators: Vec<_> = signers.iter()
        .map(|s| hex::encode(s.public_key()))
        .collect();
    
    // 3. Start 4 nodes
    let mut handles: Vec<NodeHandle> = Vec::new();
    for (i, _signer) in signers.iter().enumerate() {
        let config = NodeConfig {
            // unique per node: seed, listen_addr (port 0), rpc_addr (port 0), data_dir (tempdir)
            // shared: validators, namespace, block_interval (1s)
            // bootstrap_peers: wire to previously started nodes
            ..
        };
        let handle = start_node(config).await.expect("node start failed");
        // Log: "Started node {i}: rpc={}, p2p={}, pubkey={}"
        handles.push(handle);
    }

    // 4. Verification loop
    let result = timeout(Duration::from_secs(60), async {
        loop {
            let mut heights = Vec::new();
            for (i, handle) in handles.iter().enumerate() {
                let height = rpc_get_block_number(handle.rpc_addr).await;
                println!("Node {i}: height={height}");
                heights.push(height);
            }
            
            // Check: all heights > 0
            if heights.iter().all(|h| *h > 0) {
                // Check: all within ±1
                let min = heights.iter().min().unwrap();
                let max = heights.iter().max().unwrap();
                if max - min <= 1 {
                    println!("All nodes synced: heights={:?}", heights);
                    return heights;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }).await;

    assert!(result.is_ok(), "Timeout: nodes did not reach consensus within 60s");
    let final_heights = result.unwrap();
    
    // 5. Assert all 4 nodes have height > 0
    for (i, h) in final_heights.iter().enumerate() {
        assert!(*h > 0, "Node {i} height should be > 0, got {h}");
    }
    
    // 6. Assert heights within ±1
    let min = final_heights.iter().min().unwrap();
    let max = final_heights.iter().max().unwrap();
    assert!(max - min <= 1, "Heights not synced: min={min}, max={max}");

    // 7. Cleanup
    for handle in handles {
        handle.join_handle.abort();
    }
}

async fn rpc_get_block_number(addr: SocketAddr) -> u64 {
    // JSON-RPC call to eth_blockNumber
    // Parse hex response to u64
}
```

### Step 3: Implement rpc_get_block_number helper
Use `jsonrpsee` client or raw HTTP POST to call eth_blockNumber.

### Step 4: Handle port discovery and bootstrap peer wiring
After each node starts, its `NodeHandle.listen_addr` has the actual bound port. Wire subsequent nodes' bootstrap_peers to include previously started nodes.

### Step 5: Run and verify
```bash
nix develop --command cargo test -p integration-tests -- --test-threads=1 test_four_node_consensus --nocapture 2>&1
```

Expected output should show:
- Each node starting with P2P layer info
- Per-node height polling: "Node 0: height=1", "Node 1: height=1", etc.
- "All nodes synced: heights=[N, N, N, N]"

### Step 6: Verify all 4 nodes' heights independently
Each node's height must be checked — not just one node as proxy. The test explicitly polls all 4.

## Post-Task Gate
- [ ] `nix develop --command cargo build` passes (full workspace)
- [ ] `nix develop --command cargo test -p integration-tests -- test_four_node_consensus` passes
- [ ] Test output shows all 4 nodes reaching height > 0
- [ ] Test output shows heights within ±1 across all nodes
- [ ] Test completes within 60s
- [ ] Evidence saved to `.sisyphus/evidence/task-4-multinode-integration-test.txt`

## Mock Boundary
None — uses real whirlpool-node instances. No mocking.
