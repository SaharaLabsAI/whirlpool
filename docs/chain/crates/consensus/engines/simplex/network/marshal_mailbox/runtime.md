# Marshal runtime wiring (`SimplexEngine::start`)

This page describes the **runtime** wiring that must happen before (and alongside) starting
Simplex consensus.

Marshal has two runtime dependencies:

1. A broadcast/buffer engine for disseminating payload bytes.
2. A backfill resolver for request/response fetching of missing payload bytes.

Both are driven by marshal-specific channels; see [`channels`](./channels.md).

## Runtime inputs

At runtime, the engine wrapper should derive:

- `broadcast: (Sender, Receiver)` from `BROADCASTER_CHANNEL`
- `backfill: (Sender, Receiver)` from `MARSHAL_CHANNEL`

Then initialize the marshal backfill resolver:

```rust
// Pseudocode — not compile-ready.

let (broadcast, backfill) = {
  let broadcast = network.register_channel(BROADCASTER_CHANNEL, cfg.marshal_net.broadcast);
  let backfill = network.register_channel(MARSHAL_CHANNEL, cfg.marshal_net.backfill);
  (broadcast, backfill)
};

// Start the request/response resolver engine on top of the backfill channel.
let (ingress_rx, resolver_mailbox) = commonware_consensus::marshal::resolver::p2p::init(
  /* ctx */,
  cfg.marshal_resolver,
  backfill,
);

let marshal_resolver = (ingress_rx, resolver_mailbox);
```

## Start order (recommended)

The recommended start order (as in Alto) is:

1) Start the broadcast/buffer engine

```rust
let buffer_handle = buffer.start(broadcast);
```

2) Start the marshal actor

Marshal must be started with:

- the `Marshaled` application wrapper (so marshal can publish/resolve in the same flow)
- a buffer mailbox/handle so it can broadcast payloads
- the marshal resolver pair `(ingress_rx, resolver_mailbox)` for backfill

```rust
let marshal_handle = marshal.start(marshaled, buffer_mailbox, marshal_resolver);
```

3) Start Simplex consensus

```rust
let consensus_handle = consensus.start(votes, certificates, resolver);
```

## Why marshal is not a consensus channel

The three Simplex consensus channels (`votes/certificates/resolver`) are for consensus messages.
Marshal channels are for **payload bytes** and must remain separate so consensus message traffic
stays small and predictable.
