# DOMAINS — Real Simplex Consensus Wiring

## Domain 1: Consensus Engine Orchestration

**Owner**: `consensus-simplex` crate
**Boundary**: Orchestrates all components to run the vendor simplex BFT engine

### Domain Model

**Entities**:
- `CommonwareEngine<A, S, N>` — top-level orchestrator. Currently holds app, sink, config, network. [PROPOSED] Will also hold runtime context and signer.
- `CommonwareConfig` — engine configuration (timeouts, buffer sizes, epoch). [PROPOSED] Extended with validator set.
- `RunningEngine` (from `consensus` crate) — lifecycle handle with shutdown/status. Grounded: `consensus::engine::RunningEngine`.

**Value Objects**:
- `FinalizationSink<B>` — stateful event handler tracking finalized height. Grounded: `consensus_simplex::sink::FinalizationSink`.
- `Mailbox<B>` — implements `Automaton` + `Relay` for vendor simplex. Grounded: `consensus_simplex::mailbox::Mailbox`.
- `MailboxActor<A>` — processes mailbox messages, delegates to ConsensusApp. Grounded: `consensus_simplex::mailbox::MailboxActor`.
- `AppAdapter<A, S, B, Sig>` — implements vendor `Application` + `VerifyingApplication` + `Reporter`. Grounded: `consensus_simplex::adapter::AppAdapter`.

### Wiring Contracts

| Source | Target | Interface | Direction |
|--------|--------|-----------|-----------|
| CommonwareEngine | Mailbox | `Automaton` + `CertifiableAutomaton` + `Relay` | Engine → Mailbox (via simplex Config) |
| Mailbox | MailboxActor | `mpsc::channel<Message>` | Mailbox → Actor (async channel) |
| MailboxActor | ConsensusApp | `propose()` / `verify()` / `genesis()` | Actor → App |
| simplex::Engine | AppAdapter | `Reporter::report(Update)` | Engine → Adapter (finalization) |
| AppAdapter | EventSink | `handle(ConsensusEvent)` | Adapter → Sink |
| CommonwareEngine | simplex::Engine | `start(vote, cert, resolver)` | Engine → Vendor |
| CommonwareEngine | NetworkProvider | `start_per_channel()` | Engine → P2P |

### Boundaries

- **Inbound**: `ConsensusEngine::start()` — the only public entry point. Takes `self`, returns `RunningEngine`.
- **Outbound**: `ConsensusApp` (propose/verify/genesis), `EventSink` (handle events), `NetworkProvider` (P2P channels).
- **Vendor**: `simplex::Engine`, `simplex::Config`, `RoundRobinElector`, `Sequential`, `Oracle` (Blocker).

## Domain 2: P2P Channel Management

**Owner**: `p2p-commonware` crate
**Boundary**: Manages discovery network registration and channel multiplexing/splitting

### Domain Model

**Entities**:
- `CommonwareNetworkProvider` — registers 3 channels on discovery network. Grounded: `p2p_commonware::provider::CommonwareNetworkProvider`.
- `OracleHandle` — wraps `Oracle<PK>` for validator management and Blocker creation. Grounded: `p2p_commonware::provider::OracleHandle`.

**Value Objects**:
- `MultiplexSender<S>` — routes sends by channel. Grounded: `p2p_commonware::MultiplexSender`.
- `MultiplexReceiver<R>` — polls all channels round-robin. Grounded: `p2p_commonware::MultiplexReceiver`.
- [PROPOSED] `PerChannelNetwork` — holds 3 separate (Sender, Receiver) pairs + network handle.

### Wiring Contracts

| Source | Target | Interface | Direction |
|--------|--------|-----------|-----------|
| CommonwareNetworkProvider | discovery::Network | `register(channel, config)` | Provider → Vendor |
| [PROPOSED] CommonwareNetworkProvider | CommonwareEngine | `start_per_channel()` → PerChannelNetwork | Provider → Engine |
| OracleHandle | Oracle | `.control(public_key)` → Blocker | Handle → Blocker |

### Boundaries

- **Inbound**: `start()` (current, multiplexed) and [PROPOSED] `start_per_channel()` (per-channel pairs)
- **Outbound**: Per-channel `(CommonwareSender, CommonwareReceiver)` pairs for simplex engine
- **Vendor**: `commonware_p2p::authenticated::discovery::Network`, `Oracle`
