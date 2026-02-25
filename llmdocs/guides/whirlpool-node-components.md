# Understanding Whirlpool Node Components

This guide explains how to use and extend the core components of the Whirlpool node.

## Dual-Trait Conformance in EmptyBlock

The `EmptyBlock` struct serves two purposes by implementing both local and vendor traits. It satisfies the `consensus::Block` trait for internal use while also implementing codec and cryptography traits from the `commonware` ecosystem.

Specifically, it implements `commonware_codec::Write` and `Read` to handle its 40-byte payload. For cryptographic operations, it implements `Digestible` and `Committable`, wrapping a SHA-256 hash of its height and parent ID. This allows the block to work seamlessly with different consensus and networking layers.

## EmptyBlockApp Verification Rules

The `EmptyBlockApp` enforces five rules during block verification to ensure chain integrity:

1.  Height increment. The new block's height must be exactly one greater than its parent's height.
2.  Parent ID match. The parent ID stored in the new block must match the actual ID of its parent.
3.  No self-reference. Except for the genesis block, a block cannot have an ID that matches its parent's ID.
4.  Genesis parent zero. A block at height 0 must have its parent ID set to 32 zero bytes.
5.  Implicit genesis validity. The logic that governs genesis block creation ensures it remains valid under these rules.

## Height Tracking with FinalizationSink

The `FinalizationSink` tracks the current finalized height using an `Arc<AtomicU64>`. When a block becomes finalized, the sink updates this atomic value using `SeqCst` ordering and logs the finalized block ID. It also handles pre-finalized events and faults, logging warnings when it identifies offending nodes. This allows other parts of the system to read the current height safely without complex locking.

## Mailbox and MailboxActor Pattern

The node uses an actor pattern for handling consensus state. The `Mailbox` struct contains a channel sender to send `Message` variants to the `MailboxActor`. The actor runs a loop that processes these messages, which include requests for genesis info, block proposals, and verification results.

Responses return through `oneshot` channels, ensuring a clean separation between the requestor and the state management logic. This pattern helps manage concurrency and state transitions in a predictable way.

## Completing the Implementation

The current node implementation includes several placeholders that require additional work:

### wire.rs
This module is currently a stub. It spawns a simple thread that polls a running flag. To complete it, you must wire the simplex engine, P2P networking layer, mailbox actor, and app adapter together. This will create a functional consensus engine that can communicate with other nodes.

### main.rs
The entry point currently only prints a message to the console. It lacks tracing setup, engine initialization, Ctrl-C signal handling, and proper shutdown logic. Completing this file will turn the project into a usable binary that can be launched and managed as a service.
