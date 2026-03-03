# INTENT — Real Simplex Consensus Wiring

## Objective

Replace the stub implementation in `CommonwareEngine::start()` with real wiring that connects the already-built adapter components (Mailbox, MailboxActor, AppAdapter, FinalizationSink) to the vendor `commonware_consensus::simplex::Engine`, enabling whirlpool-node to produce real EVM blocks through BFT consensus.

## Scope

### In-Scope

- **consensus-simplex/src/engine.rs**: Replace stub `start()` with real simplex engine wiring
- **p2p-commonware/src/provider.rs**: Expose per-channel `(Sender, Receiver)` pairs (the vendor simplex engine requires 3 separate channel pairs, not multiplexed)
- **whirlpool-node/src/main.rs**: Pass additional runtime context (commonware runtime context, blocker, validator set) needed by the real engine
- **consensus-simplex/src/config.rs**: Extend `CommonwareConfig` if new fields are needed (e.g., signer, validators)
- Update existing tests in consensus-simplex to verify real wiring behavior

### Out-of-Scope

- Changes to the EVM execution layer (app-evm) — already fully implemented
- Changes to the application adapter layer (app) — already fully implemented
- Multi-node P2P networking / bootstrapping (single-validator dev mode is sufficient)
- Transaction submission API (separate concern)
- Persistent storage (in-memory is fine for now)
- Changes to vendor code

### Prior Art

- **evm-tx-execution** (Sub-Intent 1): Implemented `EvmApplication` propose/verify — COMPLETE
- **evmblock-txsource** (Sub-Intent 2): Implemented `InMemoryTxPool` — COMPLETE
- **This design** is effectively Sub-Intent 3 of the broader "produce real EVM blocks" goal

## Success Criteria

1. `CommonwareEngine::start()` creates and wires a real `simplex::Engine` instance (no stub thread)
2. The vendor simplex engine receives 3 separate P2P channel pairs (vote, certificate, resolver)
3. The mailbox actor is spawned and bridges consensus app calls through the Mailbox→MailboxActor channel
4. AppAdapter is wired as the Reporter, forwarding finalization events to the EventSink
5. `RunningEngine` shutdown cleanly aborts the simplex engine handle and all spawned tasks
6. Single-validator mode produces blocks (genesis → propose → finalize cycle completes)
7. `whirlpool-node` binary output shows real block finalization (not "stub mode - simulating finalization")
8. All existing tests pass; new integration test verifies at least one real block is finalized

## Assumptions

- Single-validator mode is sufficient (the node is both proposer and voter)
- `ed25519` is the cryptographic scheme (matches existing signer usage in main.rs)
- `RoundRobinElector` from vendor is appropriate for leader election
- `Sequential` strategy from `commonware_parallel` is appropriate
- The commonware `tokio` runtime context is available and can be threaded to the engine
- In-memory state (no WAL/persistent storage) is acceptable for now
