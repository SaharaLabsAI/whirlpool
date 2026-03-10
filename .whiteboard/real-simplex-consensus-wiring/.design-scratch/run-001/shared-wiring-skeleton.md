# Shared Wiring Skeleton

## Engine Start Flow [PROPOSED]

```
CommonwareEngine::start(self, context, blocker, validators):
  1. network.start() → per-channel (vote_sender, vote_receiver), (cert_sender, cert_receiver), (resolver_sender, resolver_receiver)
  2. Create mailbox channel: mpsc::channel(config.mailbox_size) → (tx, rx)
  3. Create Mailbox<Block>::new(tx) → implements Automaton + Relay
  4. Create MailboxActor::new(rx, height, app) → spawn task via context
  5. Create AppAdapter::new(app, sink) → implements Application + VerifyingApplication + Reporter
  6. Create FinalizationSink::new(height) → tracks height (EXISTING)
  7. Configure simplex::Config with:
     - scheme: ed25519 signer
     - elector: RoundRobinElector from validators
     - blocker: Oracle from p2p discovery
     - automaton: Mailbox
     - relay: Mailbox (clone)
     - reporter: AppAdapter
     - strategy: Sequential
     - timing params from CommonwareConfig
  8. Create simplex::Engine::new(context, config)
  9. engine.start(vote_channels, cert_channels, resolver_channels) → Handle<()>
  10. Return RunningEngine with shutdown tied to Handle abort
```

## P2P Channel Splitting [PROPOSED]

Current: `NetworkProvider::start()` → `(MultiplexSender, MultiplexReceiver)`
Needed: 3 separate `(Sender, Receiver)` channel pairs

Options:
A. Expose raw per-channel pairs from NetworkProvider (change trait or concrete type)
B. Add a `into_channels()` method that destructures MultiplexSender/Receiver
C. Have engine.rs work with concrete CommonwareNetworkProvider directly, bypassing trait

## Runtime Context Threading [PROPOSED]

Current: `main.rs` creates `tokio::Runner.start(|context| ...)` and passes context to network builder.
Needed: Engine also needs context for spawning MailboxActor and simplex engine.

Options:
A. Pass context through CommonwareEngine::new() or start()
B. Engine creates its own sub-context
C. Use tokio spawning directly (may break commonware runtime assumptions)
