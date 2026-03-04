# Understanding Whirlpool Node Components

## Overview

The Whirlpool node uses a modular architecture where business logic (blocks and app rules) is separated from consensus plumbing (engine) and networking (provider). This guide explains how to wire these components together.

## Network Provider Construction

The node uses the `p2p-commonware` crate to bridge to the Commonware P2P stack. Construction follows a builder pattern:

1.  **Initialize Builder**: Provide the signer (private key) and a unique namespace.
2.  **Configure Network**: Set the listen address, dialable address, and any bootstrapper nodes.
3.  **Build**: Pass the runtime context (e.g. `tokio::Runner` context) to `build()`. This returns the `CommonwareNetworkProvider` and an `OracleHandle`.
4.  **Seed Validators**: Use the `OracleHandle` to set the initial validator set for the network.

Example:
```rust
let (provider, mut oracle_handle) = CommonwareNetworkProviderBuilder::new(signer, NAMESPACE)
    .listen_addr(listen_addr)
    .dialable_addr(dial_addr)
    .bootstrappers(bootstrappers)
    .build(context);

oracle_handle.update_validators(0, validators).await;
```

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

The node consumes the `CommonwareEngine` API from consensus-simplex:

1. Create an `EmptyBlockApp` instance
2. Create an `EventSink` implementation (e.g., FinalizationSink imported from consensus-simplex)
3. Construct `CommonwareEngine::new(app, sink, config)`
4. Call `engine.start()` to spawn consensus tasks and return `RunningEngine`
5. Query `running.height()` for current finalized height
6. Call `running.shutdown()` for graceful shutdown

This sealed API simplifies the node to pure business logic \u2014 all consensus wiring is internal to consensus-simplex.

## Architecture Evolution

Previously, whirlpool-node contained Mailbox, FinalizationSink, and Wire modules. These have been moved to consensus-simplex as generic types:
- Mailbox: Generic over block type, implements Automaton for simplex engine
- FinalizationSink: Generic EventSink tracking finalized height
- Engine wiring: Sealed in CommonwareEngine constructor \u2014 no starter closures

The node now focuses entirely on:
- EmptyBlock (dual-trait block definition)
- EmptyBlockApp (verification rules)
- Config constants (NAMESPACE, BLOCK_INTERVAL, VALIDATOR_SEED, BIND_ADDR)
