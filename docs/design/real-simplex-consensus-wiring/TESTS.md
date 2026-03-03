# TESTS — Real Simplex Consensus Wiring

## Unit Tests

### consensus-simplex/src/engine.rs

| Test | Description | Verifies |
|------|-------------|----------|
| `test_engine_can_be_constructed` | Create CommonwareEngine with mock app, sink, network | Constructor accepts all component types |
| `test_engine_starts_with_real_simplex` | Start engine with single-validator config, verify it returns RunningEngine | Real simplex::Engine is created and started (no stub thread) |
| `test_engine_shutdown_aborts_handle` | Start engine, call shutdown, verify clean exit | Handle abort stops all actors and returns |
| `test_engine_status_tracks_height` | Start engine, wait for finalization, check height > 0 | FinalizationSink updates height on real finalization events |

### p2p-commonware/src/provider.rs

| Test | Description | Verifies |
|------|-------------|----------|
| `test_start_per_channel_returns_three_pairs` | Call start_per_channel, verify 3 (Sender, Receiver) pairs | Channel registration and splitting works correctly |
| `test_per_channel_send_receive` | Send on one channel pair, verify received | Per-channel routing is correct |

## Integration Tests

### consensus-simplex (integration)

| Test | Description | Verifies | Success Criteria |
|------|-------------|----------|------------------|
| `test_single_validator_produces_block` | Wire real EvmApplication + CommonwareEngine in single-validator mode, wait for block finalization | Full propose→verify→finalize cycle with real EVM execution | Height reaches >= 1 within 30 seconds |
| `test_single_validator_with_transactions` | Submit txs to InMemoryTxPool, start engine, verify blocks contain txs | Transactions flow through propose→execute→finalize | Finalized block has non-empty transactions |

## Cross-Crate Test Seams

| Boundary | Real | Mocked | Justification |
|----------|------|--------|---------------|
| ConsensusApp | EvmApplication (integration) / MockApp (unit) | MockApp for unit tests | Unit tests shouldn't depend on reth |
| EventSink | FinalizationSink (all tests) | Real — lightweight | No external dependencies |
| NetworkProvider | MockNetworkProvider (unit) | Mock for unit tests | Unit tests shouldn't need real P2P |
| simplex::Engine | Real (integration) | N/A (mocked at Mailbox level for unit) | Integration tests verify real simplex wiring |
| State DB | InMemoryStateDb (integration) | N/A | Already in-memory, no mock needed |

## Acceptance Criteria Mapping

| INTENT Success Criterion | Test(s) |
|--------------------------|---------|
| 1. Real simplex::Engine instance | `test_engine_starts_with_real_simplex` |
| 2. 3 separate P2P channel pairs | `test_start_per_channel_returns_three_pairs` |
| 3. Mailbox actor spawned | `test_single_validator_produces_block` (implicit — propose works) |
| 4. AppAdapter wired as Reporter | `test_engine_status_tracks_height` (height updates from Reporter) |
| 5. Clean shutdown | `test_engine_shutdown_aborts_handle` |
| 6. Single-validator block production | `test_single_validator_produces_block` |
| 7. No "stub mode" in output | `test_engine_starts_with_real_simplex` (verify no stub tracing) |
| 8. Real block finalization | `test_single_validator_produces_block`, `test_single_validator_with_transactions` |
