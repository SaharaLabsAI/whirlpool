# STRATEGY — Real Simplex Consensus Wiring

## Architecture Direction

Replace the stub consensus loop with real vendor simplex engine wiring. The adapter stack (Mailbox, MailboxActor, AppAdapter, FinalizationSink) is already built and tested — the work is purely about wiring these components to the vendor `simplex::Engine` and threading the required runtime/network resources.

## Key Decisions

### D1: P2P Channel Exposure — Bypass NetworkProvider Trait [PROPOSED]

**Problem**: Vendor `simplex::Engine::start()` needs 3 separate `(Sender, Receiver)` channel pairs (vote, certificate, resolver). Current `NetworkProvider` trait returns multiplexed channels.

**Decision**: Add a `start_per_channel()` method to `CommonwareNetworkProvider` (concrete type, not the trait) that returns `PerChannelNetwork` containing the 3 individual channel pairs plus the network handle. The engine will accept the concrete provider type rather than the generic trait.

**Rationale**: Changing the `NetworkProvider` trait would impact all implementations (including mock). The simplex engine already depends on concrete commonware types, so concrete typing is acceptable. The generic trait remains for other consumers.

**Alternative considered**: Destructuring MultiplexSender/MultiplexReceiver back into per-channel pairs. Rejected because it would require unsafe or complex interior extraction from opaque wrappers.

### D2: Runtime Context Threading — Pass as Argument [PROPOSED]

**Problem**: Vendor `simplex::Engine::new()` requires a commonware runtime context (`E: Clock + CryptoRngCore + Spawner + Storage + Metrics`). Currently only main.rs has access to the runtime context.

**Decision**: Add a runtime context parameter to `CommonwareEngine::start()` or `new()`. The engine needs the context to:
1. Spawn the MailboxActor task
2. Create and start the vendor simplex engine
3. Provide Clock/Metrics to simplex internals

**Signature change**: `CommonwareEngine::start(self, context: C)` where `C` is the commonware tokio runtime context type.

### D3: Validator Set — Single-Validator Dev Mode [PROPOSED]

**Problem**: Simplex BFT needs a validator set for leader election and quorum. In dev mode, there's only one validator.

**Decision**: Pass the validator's public key as part of engine configuration. `CommonwareConfig` gains a `validators: Vec<PublicKey>` field. The `RoundRobinElector` is constructed from this set. For dev mode, the set contains exactly one validator (the node's own key).

### D4: Blocker — From P2P Oracle [PROPOSED]

**Problem**: Vendor `simplex::Config` needs a `Blocker` implementation. The `Oracle` from `commonware_p2p::authenticated::discovery` implements `Blocker`.

**Decision**: The oracle handle (already returned by `CommonwareNetworkProviderBuilder::build()`) provides `.control(public_key)` which returns an Oracle implementing Blocker. Pass this to the engine configuration. May require exposing the oracle handle through a new method on the provider or passing it alongside.

### D5: Shutdown Model — Handle Abort [PROPOSED]

**Problem**: Current stub uses `AtomicBool` + thread polling for shutdown. Real simplex engine returns `Handle<()>` which can be aborted.

**Decision**: `RunningEngine` stores the simplex `Handle<()>` and aborts it on shutdown. The MailboxActor and other spawned tasks will be cancelled when their runtime context is dropped or aborted.

### D6: ConsensusEngine Trait — Extend start() or Restructure [PROPOSED]

**Problem**: Current `ConsensusEngine::start(self)` takes no arguments, but the real implementation needs a runtime context. Changing the trait signature impacts all implementations.

**Decision**: Change `ConsensusEngine::start()` to not require additional parameters. Instead, store the runtime context in `CommonwareEngine` at construction time. This keeps the trait generic. `CommonwareEngine::new()` gains a context parameter.

## Risk Areas

| Risk | Impact | Mitigation |
|------|--------|------------|
| Vendor simplex::Engine type parameters are complex (9 generics) | High complexity in engine.rs | Use concrete types (ed25519, Sequential, RoundRobinElector) to reduce generic sprawl |
| Runtime context lifetime/ownership with commonware runtime | May conflict with tokio task spawning | Test with commonware_runtime::tokio context; use context.with_label() for isolation |
| Single-validator BFT may have edge cases | May not produce blocks if quorum logic expects n>1 | Test empirically; simplex is designed for n=1 case |
| P2P channels may not work in loopback | Network discovery may fail without real peers | Use localhost with self-connection; verify in integration test |
| MailboxActor verify is permissive (accepts any valid digest) | Security concern for multi-validator | Acceptable for single-validator dev; flag for future hardening |

## Ordering

1. **p2p-commonware**: Add `start_per_channel()` method exposing individual channel pairs
2. **consensus-simplex/config**: Extend `CommonwareConfig` with signer/validators if needed
3. **consensus-simplex/engine**: Replace stub with real simplex engine wiring
4. **whirlpool-node/main.rs**: Update to pass runtime context and additional config
5. **Tests**: Update existing tests, add integration test for real block finalization

## Open Questions (Triage)

| Question | Type | Status |
|----------|------|--------|
| Does simplex work correctly with n=1 validator? | `information-gap` | `UNKNOWN` — test empirically |
| Does commonware tokio runtime context implement Storage trait? | `information-gap` | `UNKNOWN` — check vendor; may need in-memory storage wrapper |
| What buffer_pool (PoolRef) value is appropriate? | `information-gap` | `UNKNOWN` — check vendor defaults/tests |
