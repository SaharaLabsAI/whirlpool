# Understanding Whirlpool Node Components

## Overview

The Whirlpool node uses a modular architecture where business logic (blocks and app rules) is separated from consensus plumbing (engine) and networking (provider). This guide explains how to wire these components together.

## Network Provider Construction

The node uses the `p2p-commonware` crate to bridge to the Commonware P2P stack. Construction follows a builder pattern:

1.  **Initialize Builder**: Provide the signer (private key) and a unique namespace.
2.  **Configure Network**: Set the listen address, dialable address, and any bootstrapper nodes.
3.  **Seed Initial Validators**: Pass the startup validator list into `.initial_validators(0, validators.clone())` before calling `build(...)`.
4.  **Build**: Pass the runtime context (e.g. `tokio::Runner` context) to `build()`. This returns the `CommonwareNetworkProvider` and an `OracleHandle`.

Example:
```rust
let args = NodeArgs::parse();
let config = NodeConfig::from(args);
let validators = config
    .validators
    .clone()
    .unwrap_or_else(|| vec![signer.public_key()]);

let (provider, oracle_handle) = CommonwareNetworkProviderBuilder::new(signer, config.network.namespace.clone())
    .listen_addr(config.network.listen_addr)
    .dialable_addr(config.network.dialable_addr)
    .max_message_size(config.network.max_message_size)
    .initial_validators(0, validators.clone())
    .bootstrappers(config.network.bootstrap_peers.clone())
    .build(context);
```

Today the same `validators` list also feeds `CommonwareConfig.validators`, so it seeds both the P2P discovery/oracle path and the simplex engine membership at startup (`crates/node/src/node.rs:83` and `crates/node/src/node.rs:151`).

## Dual-Trait Conformance in EmptyBlock

This guide explains the core components of the Whirlpool node after the consensus wiring refactor.

## Dual-Trait Conformance in EmptyBlock

The `EmptyBlock` struct serves two purposes by implementing both local and vendor traits. It satisfies the `consensus::traits::Block` trait for internal use while also implementing codec and cryptography traits from the `commonware` ecosystem.

Specifically, it implements `commonware_codec::Write` and `Read` to handle its 40-byte payload. For cryptographic operations, it implements `Digestible` and `Committable`, wrapping a SHA-256 hash of its height and parent ID. This allows the block to work seamlessly with different consensus and networking layers.

## EmptyBlockApp Verification Rules

The `EmptyBlockApp` enforces five rules during block verification to ensure chain integrity:

1.  Height increment. The new block's height must be exactly one greater than its parent's height.
2.  Parent ID match. The parent ID stored in the new block must match the actual ID of its parent.
3.  No self-reference. Except for the genesis block, a block cannot have an ID that matches its parent's ID.
4.  Genesis parent zero. A block at height 0 must have its parent ID set to 32 zero bytes.
5.  Implicit genesis validity. The logic that governs genesis block creation ensures it remains valid under these rules.

## Using the Node

The node library provides a high-level API in `whirlpool_node::node`:

1.  **Parse Arguments**: Use `NodeArgs::parse()`.
2.  **Load Configuration**: Call `load_config(args)` to merge CLI and TOML (crates/node/src/config.rs:319).
3.  **Start Node**: Call `start_node(config)` to launch consensus and RPC (crates/node/src/node.rs:50).
4.  **Manage Lifecycle**: Use the `NodeHandle` for monitoring and teardown on `Drop` (crates/node/src/node.rs:26).

## Architecture Evolution

Whirlpool components are now separated into distinct modules for better reuse:
- **config**: Multi-source configuration layering (CLI > TOML > Defaults).
- **node**: Programmatic node lifecycle management (start/stop/handle).
- **main**: Minimalist binary wrapper.
