# consensus-simplex — Contract Document

## Purpose

Wire the vendor `commonware_consensus::simplex::Engine` to the already-built adapter components (Mailbox, MailboxActor, AppAdapter, FinalizationSink), replacing the stub loop in `CommonwareEngine::start()`.

## Public Interface Changes

### Modified: `CommonwareEngine<A, S, N>` [PROPOSED]

**Current** (Grounded: `crates/consensus-simplex/src/engine.rs`):
```rust
pub struct CommonwareEngine<A, S, N> {
    app: Arc<A>,
    sink: Arc<S>,
    config: CommonwareConfig,
    network: N,
}

impl CommonwareEngine<A, S, N> {
    pub fn new(app: Arc<A>, sink: Arc<S>, config: CommonwareConfig, network: N) -> Self
}

impl ConsensusEngine for CommonwareEngine<A, S, N> {
    fn start(self) -> Result<RunningEngine, ConsensusError>
}
```

**Proposed**:
```rust
pub struct CommonwareEngine<A, S, N, C> {
    app: Arc<A>,
    sink: Arc<S>,
    config: CommonwareConfig,
    network: N,
    context: C,  // [PROPOSED] commonware runtime context
}

impl CommonwareEngine<A, S, N, C> {
    pub fn new(app: Arc<A>, sink: Arc<S>, config: CommonwareConfig, network: N, context: C) -> Self
}
```

The `start()` method body changes from stub to real wiring, but the return type stays `Result<RunningEngine, ConsensusError>`.

### Modified: `CommonwareConfig` [PROPOSED]

**Current** (Grounded: `crates/consensus-simplex/src/config.rs`):
- Timing/buffer fields: `namespace`, `leader_timeout`, `notarization_timeout`, etc.

**Proposed additions**:
- `signer`: ed25519 private key (for simplex scheme)
- `validators`: `Vec<PublicKey>` (for elector construction)

## Internal Changes

### `start()` method — replace stub (lines 81-157)

Remove:
- Unused Mailbox/MailboxActor/FinalizationSink creation (lines 94-103)
- Stub thread spawning (lines 115-138)

Add:
- Real P2P channel setup via `network.start_per_channel()`
- Mailbox + MailboxActor creation and spawn
- AppAdapter creation
- FinalizationSink creation
- `simplex::Config` assembly with concrete types
- `simplex::Engine::new(context, config).start(vote, cert, resolver)` → Handle
- RunningEngine wrapping Handle abort as shutdown

## Dependencies

- **New vendor deps**: `commonware_consensus::simplex` (Engine, Config), `commonware_parallel::Sequential`, `commonware_p2p::Blocker`
- **Existing**: `consensus` (traits), `p2p` (NetworkProvider), all adapter modules

## Risks

- 9-generic vendor Config type requires careful type threading
- Runtime context ownership (move into Engine::new, consumed by simplex start)
