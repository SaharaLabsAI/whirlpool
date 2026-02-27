# Wiring: State Storage

<!-- continuation round 2 -->

This document describes how the `state` crate connects to other crates in the Whirlpool EVM integration. It focuses on the concrete implementation of the REVM database and its lifecycle within the node.

## Wiring matrix

The following table maps the wiring edges where the `state` crate provides capabilities consumed by other components.

| Provider | Capability | Consumer | Mechanism |
|----------|-----------|----------|-----------|
| `state` | `revm::Database` impl (`InMemoryStateDb`) | `app-evm` (`EvmApplication<DB>`) | Generic type parameter `DB` instantiated with `InMemoryStateDb` [PROPOSED] |
| `state` | `commit(bundle_state: &BundleState)` | `app-evm` (`EvmApplication`) | Direct method call after block execution [PROPOSED] |
| `state` | `state_root() -> B256` | `app-evm` (`EvmApplication`) | Called after commit to get new state root for block header [PROPOSED] |
| `state` | `Clone` (snapshot) | `app-evm` | Clone state before speculative execution, discard on verification failure [PROPOSED] |
| `state` | `genesis state initialization` | `whirlpool-node` | Node startup: create InMemoryStateDb with genesis accounts [PROPOSED] |

## Wiring resolution for B-002

The blocker **B-002 (State DB generic)** identified in the initial design is resolved by the following wiring patterns:

1. **Generic Resolution**: The `EvmApplication<DB: Database + Clone>` defined in the execution domain is now concretely instantiated with `DB = InMemoryStateDb` from the `state` crate.
2. **Node Lifecycle**: 
   - At startup, the node creates a new state instance: `InMemoryStateDb::new()` or `InMemoryStateDb::with_genesis(genesis_alloc)`.
   - This owned instance is passed into the `EvmApplication` constructor.
3. **Execution Flow (Propose)**:
   - The application clones the current state to create a speculative working copy.
   - Transactions are executed against this clone.
   - Upon successful execution, the resulting `BundleState` and computed `state_root()` are used to assemble the block.
   - The clone is **not** committed to canonical state during propose — it is held pending finalization.
4. **Verification Flow (Verify)**:
   - The application clones the state before re-executing a received block.
   - If verification succeeds (state roots match), the `BundleState` is held pending finalization.
   - If verification fails, the clone is discarded, leaving canonical state untouched.
5. **Finalization (Commit)**:
   - When consensus finalizes a block, the pending `BundleState` is committed to canonical state via `Arc<RwLock<InMemoryStateDb>>`.
   - This ensures non-finalized blocks never corrupt canonical state, supporting forks and rollbacks.

## Component Integration Example

The following pseudo-code illustrates how the node wires these components together during initialization:

```rust
// [PROPOSED] Node startup wiring in whirlpool-node
fn bootstrap_node(config: WhirlpoolConfig) -> WhirlpoolNode {
    // 1. Initialize state with genesis
    let genesis = load_genesis(&config.chain_spec);
    let state_db = InMemoryStateDb::with_genesis(genesis);
    
    // 2. Resolve generic DB type for the EVM application
    let evm_app = EvmApplication::<InMemoryStateDb>::new(
        config.evm,
        Arc::new(RwLock::new(state_db))  // shared ownership for finalization
    );
    
    // 3. Wrap in consensus adapter
    let consensus_app = ApplicationAdapter::new(evm_app);
    
    WhirlpoolNode::new(consensus_app)
}
```

## Implementation Details

The `InMemoryStateDb` serves as a wrapper around REVM's `BundleState` and internal storage, providing the necessary `Database` and `DatabaseCommit` traits. It is designed for high-performance in-memory execution without immediate disk I/O, supporting the Whirlpool MVP requirements.

### State Snapshots

Snapshots are achieved through the standard `Clone` trait. Since the state is currently in-memory and utilizes efficient data structures, cloning is a viable mechanism for speculative execution boundaries.

## Open wiring questions

- ~~[BLOCKER B-001] **ChainSpec still unresolved**~~ — Resolved (round 3). Genesis allocation is empty (`Default::default()`). Chain ID `313_371`, Cancun-activated. `InMemoryStateDb::with_genesis` receives `chain_spec.genesis.alloc.clone()`. <!-- continuation round 3: B-001 resolved -->
- **State persistence**: Persistence across node restarts is currently out of scope for the MVP. The wiring assumes an in-memory database that is re-initialized from genesis or a snapshot at each start.
