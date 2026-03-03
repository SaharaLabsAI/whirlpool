# Design Contract Table: real-simplex-consensus-wiring

## Scope Boundaries
### In-Scope
- Replace stub consensus loop with real wiring in `CommonwareEngine::start()` to vendor `commonware_consensus::simplex::Engine` (consensus-simplex).
- Expose 3 separate P2P `(Sender, Receiver)` pairs via `CommonwareNetworkProvider::start_per_channel()` (p2p-commonware).
- Update `whirlpool-node` binary wiring to pass runtime context + validator config + blocker/oracle handle to the engine.
- Extend `consensus-simplex` `CommonwareConfig` if needed (e.g., signer, validators).
- Update/add tests to verify real wiring and single-validator block finalization.

### Out-of-Scope
- EVM execution layer changes (`app-evm`).
- Application adapter changes (`app`).
- Multi-node P2P networking/bootstrapping (single-validator dev mode is sufficient).
- Transaction submission API.
- Persistent storage (in-memory acceptable).
- Changes to vendor code.

## Crate Ownership
| Crate | Owns | Does NOT Own |
|---|---|---|
| `consensus-simplex` | Simplex engine orchestration/wiring (`CommonwareEngine`, `CommonwareConfig`), Mailbox/MailboxActor bridging, `AppAdapter` reporter wiring, `FinalizationSink` height tracking, starting/stopping vendor `simplex::Engine` | Consensus trait definitions (`ConsensusEngine`, `ConsensusApp`, `EventSink`, `RunningEngine`), P2P provider implementation, EVM execution/app logic, vendor simplex internals |
| `p2p-commonware` | Commonware discovery network registration; per-channel channel exposure (`start_per_channel()` + `PerChannelNetwork`); oracle handle access for blocker creation | Consensus engine wiring; application propose/verify logic; generic P2P traits (`p2p` crate) |
| `whirlpool-node` | Binary-only wiring: construct engine with runtime context; configure signer/validators; keep oracle handle alive and plumb blocker | Consensus engine internals; P2P provider internals; EVM execution logic |
| Adjacent (read-only): `consensus`, `app`, `app-evm`, `p2p`, `state` | Traits + already-implemented app/EVM/state pieces | Not modified by this design |

## Public Interfaces & Key Types
| Type/Trait | Crate | File | Role |
|---|---|---|---|
| `ConsensusEngine`, `ConsensusApp`, `EventSink`, `RunningEngine` | `consensus` | (design: adjacent crate; file not specified) | Core consensus traits + lifecycle handle |
| `CommonwareEngine<A, S, N>` (and proposed `CommonwareEngine<A,S,N,C>` with `context`) | `consensus-simplex` | `crates/consensus-simplex/src/engine.rs` | Top-level orchestrator; `start()` is the sole public entry point |
| `CommonwareConfig` (+ proposed `signer`, `validators`) | `consensus-simplex` | `crates/consensus-simplex/src/config.rs` | Engine configuration (timing/buffers/namespace + signer/validator set) |
| `Mailbox<B>` | `consensus-simplex` | `consensus_simplex::mailbox::Mailbox` | Implements vendor `Automaton` + `CertifiableAutomaton` + `Relay`; forwards to actor via mpsc |
| `MailboxActor<A>` | `consensus-simplex` | `consensus_simplex::mailbox::MailboxActor` | Receives mailbox messages; calls `ConsensusApp::{genesis,propose,verify}` |
| `AppAdapter<A, S, B, Sig>` | `consensus-simplex` | `consensus_simplex::adapter::AppAdapter` | Implements vendor `Application` + `VerifyingApplication` + `Reporter`; emits `ConsensusEvent` to sink |
| `FinalizationSink<B>` | `consensus-simplex` | `consensus_simplex::sink::FinalizationSink` | Tracks finalized height (shared atomic) |
| `CommonwareNetworkProvider` | `p2p-commonware` | `p2p_commonware::provider::CommonwareNetworkProvider` | P2P provider for commonware discovery network |
| `OracleHandle` | `p2p-commonware` | `p2p_commonware::provider::OracleHandle` | Provides `.control(public_key)` → `Oracle` used as vendor `Blocker` |
| `MultiplexSender<S>`, `MultiplexReceiver<R>` | `p2p-commonware` | `p2p_commonware::{MultiplexSender, MultiplexReceiver}` | Existing multiplexed channel routing/polling (kept) |
| `PerChannelNetwork<S, R>` (proposed) | `p2p-commonware` | `crates/p2p-commonware/src/provider.rs` | Holds 3 separate channel pairs + `network_handle` |
| `CommonwareNetworkProvider::start_per_channel()` (proposed) | `p2p-commonware` | `crates/p2p-commonware/src/provider.rs` | Registers 3 channels and returns per-channel pairs (vote/cert/resolver) |
| `commonware_consensus::simplex::{Engine, Config}` | vendor commonware | `commonware_consensus::simplex` | Vendor simplex engine + configuration |
| `RoundRobinElector` | vendor commonware | `commonware_consensus::simplex` | Leader election from `validators` |
| `Sequential` | vendor commonware | `commonware_parallel::Sequential` | Strategy used by simplex |
| `Oracle` (`Blocker`) | vendor commonware | `commonware_p2p::authenticated::discovery` | Blocker implementation sourced from oracle handle |

## Flow Requirements
### Engine Startup (replace stub)
- Call `network.start_per_channel()` to register 3 channels (VOTE=0, CERT=1, RESOLVER=2) and obtain `PerChannelNetwork { vote, certificate, resolver, handle }`.
- Create `mpsc::channel(config.mailbox_size)` and `Mailbox::<Block>::new(tx)`.
- Spawn `MailboxActor::new(rx, height.clone(), app.clone())` using the commonware runtime context.
- Create `AppAdapter::new(app.clone(), sink.clone())`.
- Create `FinalizationSink::<Block>::new(height.clone())`.
- Assemble `simplex::Config` with: ed25519 signer, `RoundRobinElector::new(validators)`, blocker via `OracleHandle.control(public_key)`, automaton+relay = `Mailbox`, reporter = `AppAdapter`, strategy = `Sequential`, timing mapped from `CommonwareConfig`.
- Start vendor engine: `simplex::Engine::new(context, config).start(vote, cert, resolver)` → `Handle<()>`.
- Return `RunningEngine` wrapping the handle; shutdown aborts the handle; status/height read from the shared atomic.

### Block Production Cycle (Propose → Verify → Finalize)
- Propose: simplex → `automaton.propose(..)` → `Mailbox` → `MailboxActor` → `ConsensusApp::propose()` → returns block/digest to simplex.
- Verify: simplex → `automaton.verify(..)` → `MailboxActor` → `ConsensusApp::verify()`; errors cause simplex to reject.
- Finalize: simplex → `reporter.report(Update::Block(..))` → `AppAdapter` → `EventSink::handle(ConsensusEvent::Finalized { .. })` → `FinalizationSink` updates height → `ack.acknowledge()`.

### Engine Shutdown
- `RunningEngine::shutdown()` aborts the vendor `Handle<()>` and sets `running=false`.
- Actor + P2P channels terminate as senders/receivers are dropped.

## Test Contracts
### Unit Tests
- `consensus-simplex/src/engine.rs`
  - `test_engine_can_be_constructed`: constructing `CommonwareEngine` with mock app/sink/network succeeds.
  - `test_engine_starts_with_real_simplex`: starting with single-validator config returns `RunningEngine` and uses real simplex (no stub thread).
  - `test_engine_shutdown_aborts_handle`: shutdown cleanly aborts the simplex handle and exits.
  - `test_engine_status_tracks_height`: after finalization, engine-reported height is `> 0` (FinalizationSink updates on real events).
- `p2p-commonware/src/provider.rs`
  - `test_start_per_channel_returns_three_pairs`: `start_per_channel()` returns 3 (Sender, Receiver) pairs.
  - `test_per_channel_send_receive`: sending on one per-channel pair is received (routing correctness).

### Integration Tests
- `consensus-simplex` (integration)
  - `test_single_validator_produces_block`: real `EvmApplication` + `CommonwareEngine` in single-validator mode completes propose→verify→finalize; height reaches `>= 1` within 30 seconds.
  - `test_single_validator_with_transactions`: submit txs to `InMemoryTxPool`; finalized block has non-empty transactions.

## Active Blockers
- None (per `docs/design/real-simplex-consensus-wiring/BLOCKERS.md`).
