# Task 05 Evidence: Provider Chain Context and Subscriptions

## Changes Made

### `crates/rpc-eth/src/provider.rs`
- Added `PlainAccountState` to `state_reth::tables` import
- Replaced noop `AccountReader::basic_account` with real MDBX-backed implementation:
  reads directly from `PlainAccountState` table via `tx.get::<PlainAccountState>(address)`

### `crates/rpc-eth/Cargo.toml`
- Added `state-reth` and `tempfile` as dev-dependencies (for test MDBX setup)

### `crates/rpc-eth/tests/provider_contract.rs`
- Extended TST-1 with 3 new runtime tests:
  - `chain_spec_provider_returns_spec` (TST-1b): verifies ChainSpecProvider returns valid chain id
  - `account_reader_returns_none_for_unknown` (TST-1c): verifies AccountReader returns None on empty DB
  - `canon_state_subscriptions_yields_receiver` (TST-1d): verifies CanonStateSubscriptions wiring
- Added `AccountReader` to type-level bounds assertion
- Added `test_provider()` helper using tempfile + RethStateDb::open

## Verification

- `nix develop --command cargo build -p rpc-eth` — ✅ passes
- `nix develop --command cargo test -p rpc-eth` — ✅ 21/21 tests pass (17 eth_handler + 4 provider_contract)

## Pre-existing Trait Status
- **Already real** (confirmed): ChainSpecProvider, CanonStateSubscriptions, NodePrimitivesProvider
- **Now real** (this task): AccountReader
- **Remaining noop stubs**: StateRootProvider, StorageRootProvider, StateProofProvider, HashedPostStateProvider, StateReader, StateProvider, BytecodeReader, StageCheckpointReader, ChangeSetReader, PruneCheckpointReader
