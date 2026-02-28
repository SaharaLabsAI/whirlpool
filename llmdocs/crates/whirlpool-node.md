# whirlpool-node: EVM Consensus Binary

## Summary

`whirlpool-node` is a consensus node binary for EVM execution on Sahara Chain. It runs the Commonware consensus engine with `EvmApplication` (Solidity contract execution via revm).

Location: `crates/whirlpool-node/`

## Dependencies (All Required)

- **consensus**: Core trait layer (Block, ConsensusEngine, ConsensusApp)
- **consensus-simplex**: Sealed adapter (CommonwareEngine, FinalizationSink, CommonwareConfig)
- **whirlpool-node** (lib): Shared exports (config, EmptyBlock, EmptyBlockApp via re-exports)
- **app-evm**: EVM application executor and state provider
- **app**: ApplicationAdapter, NoopTxSource for transaction routing
- **state**: In-memory state database
- **revm**: Ethereum Virtual Machine (execution engine)
- **alloy-primitives**: EVM primitive types (Address, B256, etc.)
- **p2p-commonware**: Commonware network provider bridge
- **commonware-runtime**: tokio-based async runtime (via commonware)

**Note**: All dependencies are unconditional (no feature gates). The `evm` feature was removed; the binary now requires EVM execution.

## Architecture: EVM-Only (No Feature Gates)

File: `crates/whirlpool-node/src/main.rs`

Key differences from non-EVM variant:

1. **Imports** (lines 1–18): All EVM imports unconditional
   - `use app_evm::executor::{EvmApplication, StateProvider}`
   - `use app_evm::{WhirlpoolEvmConfig, build_sahara_chain_spec}`
   - `use state::InMemoryStateDb`
   - NO `#[cfg(feature = "evm")]` gates

2. **State Provider** (lines 26–40): TestStateDb implements StateProvider + revm::Database
   - Wraps InMemoryStateDb
   - Exposes state_root() for EVM validation
   - Implements basic(), code_by_hash(), storage() for EVM access

3. **Application Wiring** (lines 77–142): Only EVM path
   - Creates EvmApplication with state, config
   - Removed: EmptyBlockApp fallback code path
   - Wires EVM state provider, chain spec (Sahara)

## main.rs Structure

1. **Initialization** (lines 1–23): tracing setup, constants
2. **TestStateDb impl** (lines 26–70): State provider for EVM execution
3. **fn main()** (lines 72–142):
   - Lines 77–92: FinalizationSink, EvmApplication setup
   - Lines 94–107: Network provider construction
   - Lines 109–142: Engine config, wiring, and execution loop

## Test Status

- **Unit tests**: 19 (app, block modules in lib.rs)
- **Integration tests**: 6 (network_integration.rs, single_node.rs)
- **Total**: 25 tests, all passing ✓
- **Binary builds**: ✓ (target/debug/whirlpool-node, 174M)

## Feature-Gating History

**Previous (v0)**: Binary had `[features] evm = [app-evm, state, revm, ...]`, and main.rs used `#[cfg(feature = "evm")]` to switch between EvmApplication and EmptyBlockApp.

**Current (post-refactor)**: 
- Feature section removed from Cargo.toml
- All EVM dependencies unconditional
- All `#[cfg(feature = "evm")]` guards removed
- Non-EVM code path (EmptyBlockApp in main) deleted
- Library (lib.rs) unchanged; crate still exports EmptyBlock, EmptyBlockApp for shared types

## Use Cases

- **Production consensus**: EVM contracts on Sahara Chain with Commonware consensus
- **Dapp execution**: Deploy and execute Solidity contracts in consensus blocks
- **State verification**: Track EVM state root across consensus finalization
- **Network integration**: Full peer discovery and block propagation with EVM validation

## Related Documentation

- `crates/whirlpool-node-simple.md`: Non-EVM variant (minimal consensus)
- `architecture/whirlpool-node.md`: Shared library exports (EmptyBlock, config)
- `crates/app-evm.md`: EVM application executor details
- `guides/whirlpool-node-components.md`: How to extend or modify the node
