# Gap→Task Translations: real-simplex-consensus-wiring

## Summary
- Total tasks: 5
- Complexity distribution: 2×S, 3×M, 0×L
- Waves: 3

## Test Contract IDs (derived from TESTS.md)
- **TC-001**: `test_engine_can_be_constructed`
- **TC-002**: `test_engine_starts_with_real_simplex`
- **TC-003**: `test_engine_shutdown_aborts_handle`
- **TC-004**: `test_engine_status_tracks_height`
- **TC-005**: `test_start_per_channel_returns_three_pairs`
- **TC-006**: `test_per_channel_send_receive`
- **TC-007**: `test_single_validator_produces_block`
- **TC-008**: `test_single_validator_with_transactions`

## Task List

### Task 1: Make consensus-simplex engine unit tests fail on stub and require real simplex handle wiring
- **Complexity**: S
- **Wave**: 1
- **Dependencies**: none
- **Scope**: Update existing unit tests so they fail under the current `CommonwareEngine::start()` stub and pass only when `start()` constructs and runs a real vendor simplex engine handle (and `shutdown()` aborts that handle), and height/status are sourced from the shared finalized-height mechanism.
- **Files**:
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/consensus-simplex/src/tests.rs`
- **Key types/functions**:
  - `CommonwareEngine` / `impl ConsensusEngine for CommonwareEngine` (`crates/consensus-simplex/src/engine.rs`)
  - `RunningEngine` (`crates/consensus/src/engine.rs`)
  - `FinalizationSink<B>` (`crates/consensus-simplex/src/sink.rs`)
- **Test contracts covered**: TC-001, TC-002, TC-003, TC-004
- **Acceptance criteria**:
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed`
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_starts_with_real_simplex`
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_shutdown_aborts_handle`
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_status_tracks_height`
  - `nix develop --command cargo build --workspace`
- **Mock boundary**:
  - Allowed to mock: `ConsensusApp` implementation and network/provider behavior inside unit tests
  - Must NOT mock: the real `CommonwareEngine::start()` path that calls vendor simplex start and returns a handle-backed `RunningEngine`

### Task 2: Add end-to-end consensus-simplex tests that require real propose→verify→finalize (and tx inclusion)
- **Complexity**: M
- **Wave**: 1
- **Dependencies**: none
- **Scope**: Add/extend `consensus-simplex` tests so (a) a single-validator engine run reaches height `>= 1` within 30s and (b) the “with transactions” variant observes a finalized block containing transactions. These should fail under the current stub because it does not drive real propose/verify nor surface finalized blocks/txs.
- **Files**:
  - `crates/consensus-simplex/src/tests.rs`
  - `crates/consensus-simplex/src/adapter.rs`
  - `crates/app/src/adapter.rs`
- **Key types/functions**:
  - `CommonwareEngine::start` (`crates/consensus-simplex/src/engine.rs`)
  - `AppAdapter<A, S, B, Sig>` (`crates/consensus-simplex/src/adapter.rs`)
  - `ApplicationAdapter<A>` (`crates/app/src/adapter.rs`)
  - `FinalizationSink<B>` (`crates/consensus-simplex/src/sink.rs`)
- **Test contracts covered**: TC-007, TC-008
- **Acceptance criteria**:
  - `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block`
  - `nix develop --command cargo test -p consensus-simplex -- test_single_validator_with_transactions`
  - `nix develop --command cargo build --workspace`
- **Mock boundary**:
  - Allowed to mock: none on the consensus wiring path; only harness-level timeouts/log capture
  - Must NOT mock: `CommonwareEngine`, Mailbox/MailboxActor boundary, `AppAdapter` reporter path

### Task 3: Replace `CommonwareEngine::start()` stub with real vendor simplex engine wiring
- **Complexity**: M
- **Wave**: 2
- **Dependencies**: Task 1, Task 2
- **Scope**: Implement Flow 1/3: call `CommonwareNetworkProvider::start_per_channel()` to obtain 3 `(Sender, Receiver)` pairs, build the mailbox channel and spawn `MailboxActor`, assemble vendor `simplex::Config` using the already-grounded `CommonwareConfig` (`signer`, `validators`), acquire the blocker via `OracleHandle::control(public_key)`, start `commonware_consensus::simplex::Engine`, and return a `RunningEngine` that aborts the vendor handle on shutdown.
- **Files**:
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/consensus-simplex/src/mailbox.rs`
- **Key types/functions**:
  - `CommonwareEngine` / `impl ConsensusEngine for CommonwareEngine` (`crates/consensus-simplex/src/engine.rs`)
  - `CommonwareConfig` (`crates/consensus-simplex/src/config.rs`)
  - `CommonwareNetworkProvider::start_per_channel()` / `PerChannelNetwork` (`crates/p2p-commonware/src/provider.rs`)
  - `OracleHandle` (`crates/p2p-commonware/src/provider.rs`)
  - `Mailbox<B>` / `MailboxActor<A>` (`crates/consensus-simplex/src/mailbox.rs`)
  - `AppAdapter<A, S, B, Sig>` (`crates/consensus-simplex/src/adapter.rs`)
  - `FinalizationSink<B>` (`crates/consensus-simplex/src/sink.rs`)
- **Test contracts covered**: TC-002, TC-003, TC-004, TC-007
- **Acceptance criteria**:
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_starts_with_real_simplex`
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_shutdown_aborts_handle`
  - `nix develop --command cargo test -p consensus-simplex -- test_engine_status_tracks_height`
  - `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block`
  - `nix develop --command cargo test --workspace`
  - `nix develop --command cargo build --workspace`
- **Mock boundary**:
  - Allowed to mock: unit tests may use mock `ConsensusApp` and mock network/provider
  - Must NOT mock: the vendor `simplex::Engine` in real start path

### Task 4: Close Mailbox/MailboxActor gaps required for real simplex execution (single-validator)
- **Complexity**: M
- **Wave**: 3
- **Dependencies**: Task 3
- **Scope**: Remove the “simplified” behavior noted in grounding: ensure MailboxActor drives `ConsensusApp::{genesis, propose, verify}` without digest heuristics, and implement any missing `Relay` behavior (e.g., `Relay::broadcast`) currently marked as no-op so the vendor engine can progress through its internal message flow in single-validator mode.
- **Files**:
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/tests.rs`
- **Key types/functions**:
  - `Mailbox<B>` (`crates/consensus-simplex/src/mailbox.rs`)
  - `MailboxActor<A>::run` (`crates/consensus-simplex/src/mailbox.rs`)
- **Test contracts covered**: TC-007, TC-008
- **Acceptance criteria**:
  - `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block`
  - `nix develop --command cargo test -p consensus-simplex -- test_single_validator_with_transactions`
  - `nix develop --command cargo test --workspace`
  - `nix develop --command cargo build --workspace`
- **Mock boundary**:
  - Allowed to mock: `ConsensusApp` in unit tests that target mailbox/actor behavior
  - Must NOT mock: the mailbox channel boundary used by `CommonwareEngine::start()`

### Task 5: Update whirlpool-node wiring to match the real simplex engine shape (context/config/oracle)
- **Complexity**: S
- **Wave**: 3
- **Dependencies**: Task 3
- **Scope**: Bring `crates/whirlpool-node/src/main.rs` back in sync with `CommonwareEngine` construction and `CommonwareConfig` fields (`signer`, `validators`), and ensure the oracle handle is kept/plumbed so the engine can create its blocker and run without stub-mode output.
- **Files**:
  - `crates/whirlpool-node/src/main.rs`
- **Key types/functions**:
  - `fn main()` (`crates/whirlpool-node/src/main.rs`)
  - `CommonwareEngine` (`crates/consensus-simplex/src/engine.rs`)
  - `CommonwareConfig` (`crates/consensus-simplex/src/config.rs`)
  - `OracleHandle` (`crates/p2p-commonware/src/provider.rs`)
- **Test contracts covered**: TC-007 (same wiring shape, exercised in tests)
- **Acceptance criteria**:
  - `nix develop --command cargo build -p whirlpool-node`
  - `nix develop --command cargo test --workspace`
- **Mock boundary**:
  - Allowed to mock: none
  - Must NOT mock: N/A

## Dependency Matrix
| Task | Depends On | Wave |
|---|---|---|
| 1 | none | 1 |
| 2 | none | 1 |
| 3 | 1, 2 | 2 |
| 4 | 3 | 3 |
| 5 | 3 | 3 |

## Wave Assignment
- Wave 1: Task 1, Task 2
- Wave 2: Task 3
- Wave 3: Task 4, Task 5
