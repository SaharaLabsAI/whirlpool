# Codebase Grounding Map: real-simplex-consensus-wiring

## Legend
- 🔴 missing — does not exist yet
- 🟡 stub — placeholder/todo implementation
- 🟢 partial — some real logic, incomplete
- ✅ complete — fully working

## Grounding

| Design Concept | File Path | Symbol | Status | Notes |
|---|---|---|---|---|
| CommonwareEngine lifecycle | crates/consensus-simplex/src/engine.rs | `CommonwareEngine`, `impl ConsensusEngine for CommonwareEngine` | 🟡 stub | `start()` is explicitly marked stub and simulates finalization in a thread every 5s instead of wiring vendor simplex engine. |
| Simplex engine creation | crates/consensus-simplex/src/engine.rs | `simplex::Engine::new(...).start(...)` | 🔴 missing | No vendor simplex engine construction/start call exists; only comments describe future wiring. |
| Mailbox bridge | crates/consensus-simplex/src/mailbox.rs | `Mailbox<B>` | 🟢 partial | Implements `Automaton`/`CertifiableAutomaton`/`Relay`, but `Relay::broadcast` is no-op and behavior is simplified. |
| Mailbox actor | crates/consensus-simplex/src/mailbox.rs | `MailboxActor<A>::run` | 🟢 partial | Delegates to `ConsensusApp`, but propose/verify path is simplified (cached genesis parent and digest heuristic verify). |
| Finalization sink | crates/consensus-simplex/src/sink.rs | `FinalizationSink<B>` | ✅ complete | Real `EventSink` implementation updates shared finalized height and logs finalized/prefinalized/fault events. |
| AppAdapter | crates/consensus-simplex/src/adapter.rs | `AppAdapter<A, S, B, Sig>` | ✅ complete | Implements vendor `Application`, `VerifyingApplication`, and `Reporter`; emits `ConsensusEvent::Finalized` and acknowledges updates. |
| ApplicationAdapter | crates/app/src/adapter.rs | `ApplicationAdapter<A>` | ✅ complete | Implements core `ConsensusApp` over app `Application` and maps verify errors to `ConsensusError::InvalidBlock`. |
| CommonwareConfig | crates/consensus-simplex/src/config.rs | `CommonwareConfig` | ✅ complete | Contains timing/buffer fields plus `signer` and `validators` fields expected by design. |
| SimplexConfig | crates/ | `SimplexConfig` | 🔴 missing | No `SimplexConfig` symbol found under `crates/`. |
| ConsensusEngine trait | crates/consensus/src/engine.rs | `trait ConsensusEngine` | ✅ complete | Core lifecycle trait exists (`start(self) -> Result<RunningEngine, ConsensusError>`). |
| ConsensusEngine impls | crates/consensus-simplex/src/engine.rs, crates/consensus/src/mock/engine.rs | `impl ConsensusEngine for CommonwareEngine`, `impl ConsensusEngine for MockEngine` | 🟢 partial | Real-path impl exists but is stubbed; mock impl is complete for test block finalization. |
| P2P authenticated config | crates/p2p-commonware/src/provider.rs | `discovery::Config::local(...)` | 🟢 partial | Discovery config is created via `commonware_p2p::authenticated::discovery`; exact `authenticated::Config` symbol is not present. |
| Sender/Receiver adapters | crates/p2p-commonware/src/sender.rs, crates/p2p-commonware/src/receiver.rs | `CommonwareSender`, `CommonwareReceiver` | 🟢 partial | Sender adapter is functional; receiver currently hardcodes `Channel(0)` with TODO. |
| Peer discovery and registration | crates/p2p-commonware/src/provider.rs | `discovery::Network::new`, `.register(...)` | ✅ complete | Provider registers vote/certificate/resolver channels and starts discovery network handle. |
| Per-channel simplex network | crates/p2p-commonware/src/provider.rs | `PerChannelNetwork`, `start_per_channel()` | ✅ complete | Dedicated vote/cert/resolver `(Sender, Receiver)` pairs implemented and covered by provider tests. |
| RunningEngine handle | crates/consensus/src/engine.rs | `RunningEngine` | ✅ complete | Provides status/wait/shutdown over join handle and shared atomics. |
| whirlpool-node binary entry | crates/whirlpool-node/src/main.rs | `fn main()` | 🟢 partial | Entry exists and wires app/network/engine startup, but code is out of sync with current `CommonwareEngine::new(..., context)` and config fields (`signer`, `validators`). |
| whirlpool-node wire module | crates/whirlpool-node/src/wire.rs | `wire.rs` | 🔴 missing | No `wire.rs` exists in `crates/whirlpool-node/src/`. |
| p2p-commonware crate structure | crates/p2p-commonware/src/lib.rs | module exports (`provider`, `sender`, `receiver`, multiplex types) | ✅ complete | Exposes provider/builder/oracle handle and multiplex sender/receiver adapters. |
| consensus-simplex tests | crates/consensus-simplex/src/tests.rs, crates/consensus-simplex/src/engine.rs, crates/consensus-simplex/src/mailbox.rs, crates/consensus-simplex/src/sink.rs | `#[cfg(test)]` modules | 🟢 partial | Good unit coverage exists, but tests mainly validate stub/simplified behavior rather than real vendor simplex execution. |

## Key Observations
- The core bridging components (`ApplicationAdapter`, `AppAdapter`, `Mailbox`, `FinalizationSink`) are present, but `CommonwareEngine::start()` remains a stubbed lifecycle.
- The key design target (real `commonware_consensus::simplex::Engine` startup) is not yet grounded in executable code.
- `p2p-commonware` already contains the required per-channel API (`start_per_channel`) for vote/certificate/resolver streams.
- `whirlpool-node` has a concrete binary entrypoint, but current wiring does not match the latest consensus-simplex constructor/config shape.
- No `wire.rs` module exists under `crates/whirlpool-node/src/`; wiring currently lives directly in `main.rs`.
