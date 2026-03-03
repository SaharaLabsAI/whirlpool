# FLOWS — Real Simplex Consensus Wiring

## Flow 1: Engine Startup (Primary — Replace Stub)

**Trigger**: `CommonwareEngine::start(self)` called from `main.rs`
**Actors**: whirlpool-node → CommonwareEngine → P2P → simplex::Engine

### Steps

1. **P2P Channel Setup**: Call `network.start_per_channel()` [PROPOSED] to register 3 channels (VOTE=0, CERT=1, RESOLVER=2) on discovery network and get `PerChannelNetwork { vote: (S, R), certificate: (S, R), resolver: (S, R), handle }`.

2. **Mailbox Creation**: Create `mpsc::channel(config.mailbox_size)` → `(tx, rx)`. Create `Mailbox::<Block>::new(tx)` — this implements `Automaton`, `CertifiableAutomaton`, `Relay` for the vendor engine.

3. **MailboxActor Spawn**: Create `MailboxActor::new(rx, height.clone(), app.clone())`. Spawn as async task via commonware runtime context. Actor processes genesis/propose/verify messages from Mailbox channel.

4. **AppAdapter Creation**: Create `AppAdapter::new(app.clone(), sink.clone())`. This implements `Application`, `VerifyingApplication`, `Reporter` for the vendor engine.

5. **FinalizationSink**: Create `FinalizationSink::<Block>::new(height.clone())`. Tracks block height as blocks are finalized.

6. **Simplex Config Assembly**: Build `simplex::Config` with:
   - `scheme`: ed25519 signer (from engine config)
   - `elector`: `RoundRobinElector::new(validators)`
   - `blocker`: `Oracle` from P2P oracle handle (`.control(public_key)`)
   - `automaton`: `Mailbox` (clone)
   - `relay`: `Mailbox` (clone)
   - `reporter`: `AppAdapter`
   - `strategy`: `Sequential`
   - Timing params mapped from `CommonwareConfig`

7. **Vendor Engine Start**: `simplex::Engine::new(context, config).start(vote_channels, cert_channels, resolver_channels)` → returns `Handle<()>`.

8. **RunningEngine Construction**: Wrap Handle in RunningEngine with:
   - Shutdown function: aborts the Handle
   - Height: shared AtomicU64 updated by FinalizationSink
   - Running flag: set to false on shutdown

### Error Paths

- **Network start fails**: Return `ConsensusError::Other` with network error
- **Config validation fails**: `simplex::Config::assert()` panics — catch at construction
- **Engine start fails**: Return `ConsensusError::Other`

### Implementation Slice

```
Files touched:
  - crates/consensus-simplex/src/engine.rs (lines 81-157 → replace stub)
  - crates/consensus-simplex/src/config.rs (add signer/validators fields)
  - crates/p2p-commonware/src/provider.rs (add start_per_channel method)
  - crates/whirlpool-node/src/main.rs (pass context, signer, validators)
```

## Flow 2: Block Production Cycle (Propose → Verify → Finalize)

**Trigger**: Simplex engine enters new view, selects leader
**Actors**: simplex::Engine → Mailbox → MailboxActor → ConsensusApp → EvmApplication → AppAdapter → EventSink

### Steps (Leader Path)

1. **Propose**: simplex calls `automaton.propose(context)` → `Mailbox` sends `Message::Propose` to actor → `MailboxActor` calls `app.propose(parent, height)` → `EvmApplication` drains txs, executes via reth, returns `EvmBlock` → Actor computes digest, returns to simplex.

2. **Self-Verify**: simplex calls `automaton.verify(context, digest)` → similar path → `EvmApplication` re-executes txs on cloned state → verifies roots match.

3. **Finalize**: After consensus (2f+1 votes), simplex calls `reporter.report(Update::Block(block, ack))` → `AppAdapter` emits `ConsensusEvent::Finalized { block, height, proof }` → `EventSink::handle()` → `FinalizationSink` stores height → `ack.acknowledge()`.

### Error Paths

- **Empty propose**: EvmApplication returns empty block (0 txs) — valid, simplex proceeds
- **Verify mismatch**: `app.verify()` returns error → simplex rejects block
- **Finalization sink error**: Logged, ack still sent (best-effort)

## Flow 3: Engine Shutdown

**Trigger**: `RunningEngine::shutdown()` called
**Actors**: RunningEngine → Handle<()> → simplex actors → P2P network

### Steps

1. **Abort Handle**: Call `handle.abort()` on the vendor simplex Handle
2. **Set running=false**: Update AtomicBool
3. **Actor cleanup**: MailboxActor loop ends when sender side (Mailbox) is dropped
4. **Network cleanup**: P2P channels close as senders/receivers are dropped
5. **Return**: JoinHandle resolves

### Error Paths

- **Abort timeout**: Handle may take time to abort — await with timeout
- **Actor panic**: MailboxActor panic is caught by JoinHandle
