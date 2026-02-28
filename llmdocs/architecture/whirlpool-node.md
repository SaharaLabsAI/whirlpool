# Whirlpool Node Library: Shared Types & Config

## Overview

The `whirlpool-node` crate provides shared library exports for two binaries: `whirlpool-node` (EVM) and `whirlpool-node-simple` (non-EVM). It defines EmptyBlock consensus block type, EmptyBlockApp verification rules, and configuration constants.

## Dependency Graph

- **whirlpool-node** → `consensus-simplex`: Consensus engine and mailbox infrastructure.
- **whirlpool-node** → `p2p-commonware`: Builder-based network provider construction.
- **p2p-commonware** → `commonware-p2p`: Vendor P2P implementation.
- **p2p-commonware** → `p2p`: Vendor-agnostic traits.
- **whirlpool-node** → `p2p`: Vendor-agnostic traits for app/engine wiring.
- **consensus-simplex** → `p2p`: Vendor-agnostic traits.

## Type Signatures

The Whirlpool node binary provides a minimal consensus node implementation, delegating consensus wiring to the consensus-simplex library. The node focuses purely on business logic: EmptyBlock definition and EmptyBlockApp verification rules.

## Type Signatures

### block.rs
- pub type BlockId = [u8; 32];
- pub struct EmptyBlock { height: u64, parent_id: BlockId }
- pub fn EmptyBlock::genesis() -> Self;
- pub fn EmptyBlock::new(height: u64, parent_id: BlockId) -> Self;
- fn compute_id(&self) -> BlockId

### app.rs
- pub struct EmptyBlockApp;
- pub fn EmptyBlockApp::new() -> Self;
- async fn EmptyBlockApp::genesis(&self) -> EmptyBlock;
- async fn EmptyBlockApp::propose(&self, parent: &EmptyBlock, height: u64) -> Option<EmptyBlock>;
- async fn EmptyBlockApp::verify(&self, parent: &EmptyBlock, block: &EmptyBlock) -> Result<(), ConsensusError>;

### config.rs
- pub const NAMESPACE: &[u8];
- pub const BLOCK_INTERVAL: Duration;
- pub const VALIDATOR_SEED: [u8; 32];
- pub const BIND_ADDR: &str;

## Dual-Trait Conformance: EmptyBlock

| Trait | Details |
|---|---|
| consensus::Block | type Id = BlockId; id(), parent_id(), height() |
| commonware_codec::Write | Writes u64 height + [u8;32] parent_id (40 bytes) |
| commonware_codec::Read | Reads 40 bytes; type Cfg = () |
| commonware_codec::EncodeSize | Returns 40 |
| commonware_cryptography::Digestible | type Digest = sha256::Digest; wraps compute_id() |
| commonware_cryptography::Committable | type Commitment = sha256::Digest; delegates to digest() |
| commonware_consensus::Heightable | Returns Height::new(self.height) |

## EmptyBlockApp Verification Rules

1. Height increment: block.height == parent.height + 1
2. Parent ID match: block.parent_id() == parent.id()
3. No self-reference (non-genesis): block.id() != block.parent_id() unless height == 0
4. Genesis parent zero: height 0 blocks must have parent_id == [0u8; 32]
5. Implicit genesis validity: Covered by rules above

## Library Structure (Shared)

The library (crates/whirlpool-node/src/) contains:
1. **block.rs**: EmptyBlock dual-trait conformance (Block, Write, Read, EncodeSize, Digestible, Committable, Heightable)
2. **app.rs**: EmptyBlockApp trait implementations and verification rules
3. **config.rs**: Constants (NAMESPACE, BLOCK_INTERVAL, VALIDATOR_SEED, BIND_ADDR)

Both binaries (whirlpool-node main and whirlpool-node-simple main) re-export these types from their parent crate. The library itself has no features or cfg gates.

## Binary Variants (Post-Refactor)

Two binaries now use the shared library:

### whirlpool-node (EVM)
Location: `crates/whirlpool-node/src/main.rs`
- Uses EvmApplication (requires app-evm, state, revm, alloy-primitives)
- Executes Solidity contracts in consensus blocks
- Binary size: 174M
- Test status: 25 tests passing (19 unit + 6 integration)
- No feature gates; all EVM deps unconditional

### whirlpool-node-simple (Non-EVM)
Location: `crates/whirlpool-node-simple/src/main.rs`
- Uses EmptyBlockApp from whirlpool-node lib
- Pure consensus without execution layer
- Binary size: 145M
- Test status: 0 tests (binary crate only)
- No feature gates; minimal dependencies
## Integration Tests (Real Networking)

File: `tests/network_integration.rs` — uses real `CommonwareNetworkProvider` (not mock) backed by commonware discovery p2p.

Each test runs inside `tokio::Runner::default().start(|context| async { ... })` to provide the commonware runtime context required by discovery::Network.

| Test | Validates |
|------|-----------|
| test_single_node_real_network_lifecycle | Single node with real network provider starts, finalizes blocks (height >= 1 within 30s), shuts down cleanly |
| test_two_nodes_discover_and_run | Two nodes on localhost with peer discovery (oracle.update with both public keys, bootstrapper wiring). Both finalize blocks independently |
| test_real_network_graceful_shutdown | Start + immediate shutdown; sender/receiver drop without panic |

Key wiring: `ed25519::PrivateKey::from_seed(seed)` -> `CommonwareNetworkProviderBuilder::new(signer, namespace).listen_addr(..).dialable_addr(..).bootstrappers(..).build(context)` -> `(provider, oracle_handle)` -> `oracle_handle.update_validators(epoch, peers)` -> `CommonwareEngine::new(app, sink, config, provider)` -> `engine.start()`

## Test Statistics

| Module | Test Count |
|--------|-----------|
| block.rs | 8 |
| app.rs | 11 |
| tests/single_node.rs (mock) | 3 |
| tests/network_integration.rs (real) | 3 |
| **Total** | **25** |
