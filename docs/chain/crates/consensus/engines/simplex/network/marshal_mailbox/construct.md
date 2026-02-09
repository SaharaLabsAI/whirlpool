# Constructing marshal (build-time)

This page is the missing link between:

- the *concept* of marshal (payload custody)
- and how it is actually constructed when building `SimplexEngine`

The key idea: consensus only moves **commitments/digests**; marshal is what makes those digests
turn into **payload bytes**.

## What gets constructed

At build-time (inside `SimplexEngine::new(...)`), the engine constructs:

1. A long-running **marshal actor**.
2. A cloneable **marshal mailbox** (client handle).
3. A `Marshaled` application wrapper that uses the mailbox for propose/verify.

## Build-time recipe (pseudocode)

```rust
// Pseudocode — not compile-ready.

pub struct SimplexEngine<Net> {
  // ...
  marshal: commonware_consensus::marshal::Actor<...>,
  marshal_mailbox: commonware_consensus::marshal::Mailbox<...>,
  marshaled: commonware_consensus::application::marshaled::Marshaled<...>,
}

impl<Net: core::network::Network> SimplexEngine<Net> {
  pub async fn new(app: App, mut network: Net, cfg: SimplexBuildConfig) -> Result<Self> {
    // 0) Create marshal actor + mailbox.
    //
    // Marshal config is where payload-specific knobs live: retention, persistence buffers,
    // mailbox sizing, etc.
    let (marshal, marshal_mailbox, _stats) = commonware_consensus::marshal::Actor::init(
      /* ctx */,
      /* chain storage handles */,
      /* chain index handles */,
      cfg.marshal,
    ).await;

    // 1) Wrap the chain app in `Marshaled`.
    //
    // This wrapper is what Simplex uses as its `automaton`/`relay`.
    // It uses `marshal_mailbox` so it can:
    // - publish payload bytes for proposals
    // - resolve payload bytes when verifying proposals
    let marshaled = commonware_consensus::application::marshaled::Marshaled::new(
      /* ctx */,
      app,
      marshal_mailbox.clone(),
      /* epocher */,
    );

    // 2) Anything that needs block bytes after finalization should also hold a mailbox.
    // Example: an indexer/reporter that emits `FinalizedBlock`.
    let reporter_marshal = marshal_mailbox.clone();
    // reporter uses subscriptions: `reporter_marshal.subscribe(commitment)`.

    // 3) Build the consensus engine config using `marshaled`.
    // (Details omitted; see build docs.)

    // 4) Derive Simplex consensus channels (votes/certificates/resolver) from `network`.
    // 5) Derive marshal networking (broadcast + backfill) from `network`.
    // 6) Store everything in `SimplexEngine` for `start()`.

    Ok(Self { marshal, marshal_mailbox, marshaled /*, ... */ })
  }
}
```

## Where marshal is “passed into” Simplex

Marshal is not passed into `commonware_consensus::simplex::Engine::start(...)` directly.

Instead:

- `Marshaled` (constructed with the mailbox) is embedded into the Simplex config as the app-facing
  interface.
- Marshal’s **networking** (broadcast + backfill resolver) is started alongside the engine and fed
  via the engine wrapper’s runtime wiring (see [`runtime`](./runtime.md)).
