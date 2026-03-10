# Chain Binary Architecture

## 1. System boundary

The binary owns process setup and dependency wiring; consensus logic is delegated to existing crates.

- `consensus-core` provides interfaces: `Block`, `ConsensusApp`, `EventSink`, `ConsensusEngine`.
- `consensus-commonware` provides backend pieces: `CommonwareBlock`, `CommonwareConfig`, `AppAdapter`, `CommonwareEngine`.

## 2. Runtime components

1. **Config loader**
   - Reads node identity, networking, storage, and consensus timing settings.
   - Hard-codes `block_interval = 5s` for v0.

2. **EmptyBlockApp** (`consensus_core::ConsensusApp`)
   - Implements `genesis`, `propose`, and `verify`.
   - Creates blocks with empty payload.

3. **FinalizationSink** (`consensus_core::EventSink`)
   - Handles `ConsensusEvent::Finalized` in height order.
   - Persists latest finalized height and emits logs/metrics.

4. **Commonware starter**
   - Builds the backend runtime wiring and returns shutdown/join handles.
   - Passed into `CommonwareEngine::new(starter)`.

5. **Engine handle**
   - Starts with `ConsensusEngine::start`.
   - Exposes `wait` / `shutdown` through `RunningEngine`.

## 3. Binary module layout (proposed)

```text
bin/
  chain.rs                # main entrypoint
src/chain_binary/
  config.rs               # CLI/env config parsing
  block.rs                # EmptyBlock type
  app.rs                  # EmptyBlockApp (ConsensusApp impl)
  sink.rs                 # FinalizationSink (EventSink impl)
  wire.rs                 # builds CommonwareConfig + starter closure
```

## 4. Startup sequence

```rust
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cfg = load_config()?;
    let app = Arc::new(EmptyBlockApp::new(cfg.genesis_time, cfg.chain_id));
    let sink = Arc::new(FinalizationSink::new());

    let starter = build_commonware_starter(cfg, app.clone(), sink.clone())?;
    let engine = consensus_commonware::CommonwareEngine::new(starter);
    let running = consensus_core::ConsensusEngine::start(engine)?;

    running.wait().await?;
    Ok(())
}
```

## 5. Config contract (v0)

- `chain_id: String`
- `genesis_time_unix_secs: u64`
- `block_interval_secs: u64` (must be `5` in v0)
- `namespace: String`
- `leader_timeout_ms: u64`
- `notarization_timeout_ms: u64`
- `nullify_retry_ms: u64`
- `activity_timeout: u64`
- `skip_timeout: u64`
- `mailbox_size: usize`
- `replay_buffer: usize`
- `write_buffer: usize`
- `epoch: u64`
- `fetch_timeout_ms: u64`
- `fetch_concurrent: usize`

`namespace` and timeout/buffer settings are passed into `consensus_commonware::CommonwareConfig`.

## 6. Failure handling

- `propose` failures return `ConsensusError::ProposalFailed` and are logged with height context.
- Invalid incoming blocks return `ConsensusError::InvalidBlock` from `verify`.
- Sink failures should be treated as fatal so the process can restart cleanly.
- Engine task failures bubble through `RunningEngine::wait()` as `ConsensusError::Runtime`.
