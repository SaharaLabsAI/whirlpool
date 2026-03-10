# Architecture Flows

## Scope
- Sub-Intent C only: `REQ-6`, `REQ-7`, and `REQ-8`.
- Primary implementation crate: `crates/consensus-simplex`.
- Supporting crates:
  - `crates/p2p`
  - `crates/p2p-commonware`
  - `crates/whirlpool-node`
- Source verification anchors:
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/p2p/src/types.rs`
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/whirlpool-node/src/main.rs`

## Flow 1: Proposal Broadcast Over Payload Channel

```text
vendor simplex engine
  -> Automaton::propose(ctx) on Mailbox
  -> Mailbox sends Message::Propose to MailboxActor
  -> MailboxActor calls app.propose(parent, height)
  -> block returned
  -> MailboxActor::remember_block(&block)
       - compute digest
       - store block in shared BlockStore<digest, block>
  -> digest returned to vendor engine
  -> vendor engine calls Relay::broadcast(digest)
  -> Mailbox::broadcast(digest)
       - read block from shared BlockStore
       - serialize PayloadRelayMessage { digest, payload_bytes }
       - payload_sender.send(Channel::PAYLOAD / Recipients::All)
```

### Flow guarantees
- The block is inserted into `BlockStore` before relay broadcast is attempted.
- Outbound relay uses a dedicated payload transport path, not the vendor vote/certificate/resolver channels.
- Missing local payload cache does not crash the process; the relay logs and returns.

## Flow 2: Inbound Payload Receive To Verification Cache

```text
remote peer sends PayloadRelayMessage on Channel::PAYLOAD
  -> local p2p-commonware payload receiver yields raw bytes
  -> CommonwareEngine payload task reads message loop
  -> decode PayloadRelayMessage
  -> decode block payload bytes into A::Block
  -> recompute digest from decoded block
  -> compare recomputed digest with envelope.digest
       - if mismatch: drop and trace
       - if decode fails: drop and trace
  -> if valid: store block in shared BlockStore<digest, block>
```

### Flow guarantees
- Inbound payload persistence happens outside the vendor engine and without vendor code changes.
- Only digest-validated payloads enter the shared block cache.
- The same `BlockStore` type backs both locally proposed blocks and remotely received blocks.

## Flow 3: Verification Resolves Relayed Payload

```text
vendor simplex engine receives proposal digest from peers
  -> vendor engine later calls Automaton::verify(ctx, digest)
  -> Mailbox::verify(digest) forwards to MailboxActor
  -> verification path looks up digest in shared BlockStore
       - if present: app verify logic can validate the concrete block
       - if absent: verification fails or returns false
```

### Flow guarantees
- Relay activation closes the gap between receiving a digest and having the corresponding application payload available locally.
- Verification remains digest-driven at the vendor boundary.
- No additional vendor fetch/resolver redesign is required for application payload availability in this pass.

## Flow 4: Engine Startup Wiring With Additive Payload Task

```text
CommonwareEngine::start()
  -> network.start_per_channel()
       - vote pair
       - cert pair
       - resolver pair
       - payload pair
  -> create shared BlockStore
  -> create Mailbox(mailbox_tx, block_store.clone(), payload_sender)
  -> spawn MailboxActor(mailbox_rx, height, app, block_store.clone())
  -> spawn payload receive task(payload_receiver, block_store.clone())
  -> create AppAdapter(app, sink, block_store)
  -> simplex::Engine::start(vote, cert, resolver)
```

### Flow guarantees
- The vendor engine still starts with exactly three protocol channel pairs.
- Payload receive is additive background wiring owned by `consensus-simplex`.
- `AppAdapter`, `FinalizationSink`, and finalization side effects continue using the same shared block cache model.

## Flow 5: Channel Alignment Across Crates

```text
crates/p2p/src/types.rs
  -> VOTE = 0
  -> CERTIFICATE = 1
  -> RESOLVER = 2
  -> PAYLOAD = 3

crates/p2p-commonware/src/provider.rs
  -> register 0, 1, 2, 3
  -> expose payload pair on PerChannelNetwork

crates/consensus-simplex/src/engine.rs
  -> vendor engine consumes 0, 1, 2 only
  -> payload relay task consumes 3 only
```

### Flow guarantees
- Existing protocol channel IDs do not move.
- Payload traffic is strictly additive and isolated on channel `3`.
- `REQ-7` is satisfied by preserving vote/certificate/resolver alignment while introducing a dedicated payload path.

## Traceability
- `REQ-6` -> Flow 1, Flow 2, Flow 3, Flow 4
- `REQ-7` -> Flow 5
- `REQ-8` -> Flow 4 and the compatibility guarantees attached to Flows 1-4
