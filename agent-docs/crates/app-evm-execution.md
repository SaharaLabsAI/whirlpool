# app-evm-execution

## Purpose
Pure EVM runtime/config/execution crate for Whirlpool.

## Location
`crates/app/evm/execution/`

## Ownership Boundary
`app-evm-execution` now owns EVM behavior, not Sahara chain-spec construction.

### Owns
- `WhirlpoolEvmConfig`
- `EvmApplication`
- EVM tx decode/recovery helpers
- Fee-recipient runtime behavior and validation
- Constants:
  - `DEFAULT_PROPOSER_FEE_RECIPIENT`
  - `VALIDATOR_FEE_RECIPIENTS_REGISTRY`

### Does not own anymore
- `SAHARA_CHAIN_ID`
- `build_sahara_chain_spec*`
- `try_build_sahara_chain_spec*`
- public `try_simplex_validators_from_chain_spec`

Those live in `chainspec`.

## Key Runtime Notes
- `WhirlpoolEvmConfig` still derives proposer fee recipients from genesis storage at `VALIDATOR_FEE_RECIPIENTS_REGISTRY`.
- `WhirlpoolEvmConfig` now carries FullDkg envelope knobs:
  - `full_dkg_feature_enabled`
  - `full_dkg_strict_height`
  - optional `current_full_dkg_output` (`dealers`, `players`, `public_polynomial`)
- `WhirlpoolEvmConfig` carries epoch-scoped activation override data via `with_activation_players_for_epoch(epoch, players)`, but activation resolution is delegated to `evm_precompiles::validators::ValidatorActivationSchedule`.
- Precompile injection remains in `WhirlpoolEvmConfig::evm_with_env(...)` via `evm_precompiles::whirlpool_precompiles_with_validators(...)`.
- Block header `extra_data` now uses app-shared canonical envelope bytes (`RawEth` + optional `FullDkgV1`) instead of raw proposer key bytes.
- Verify path decodes `extra_data` with a height gate (`Legacy` before strict height, `Strict` at/after strict height), enforces proposer-key parity against `RawEth`, and enforces boundary-aware FullDkg/Reshare invariants.
- Propose/verify now share one executor-local next-block base-fee seam:
  - propose derives `base_fee_per_gas` through the shared helper,
  - verify rejects blocks whose `block.base_fee_per_gas` does not match the protocol-derived next-block fee before fee accounting,
  - verify-side burned-fee / priority-fee accounting now uses the derived canonical fee after the mismatch guard.
- EVM tx decode helpers now use exact EIP-2718 decoding, so padded tx bytes fail closed during both proposal pre-decode and verify decoding.
- Boundary extra-data semantics are orchestrated here but target derivation is lower-layer-owned: `evm_precompiles::epoch::EpochActivationTargets` supplies `E`, `E+1`, and `E+2`; `evm_precompiles::validators::ValidatorActivationSchedule` resolves FullDkg/Reshare players. Non-boundary blocks must not carry `ReshareV1`, and boundary verify remains fail-closed for missing/mismatched required `ReshareV1` fields when FullDkg candidate data is configured.
- Canonical extra-data composition and include/omit predicates are centralized in `canonical_extra_data.rs` (`build_canonical_extra_data`, `full_dkg_should_be_included`, activation-parity guard) and reused by propose/verify paths through `executor/mod.rs`.
- Executor layout is now directory-backed under `app/src/executor/`:
  - `mod.rs` — public façade + `Application` trait implementation wiring.
  - `header_and_decode.rs` — header projection + tx decode helpers.
  - `state_helpers.rs` — internal fee/community-pool/state helper logic.
  - `impl_core_methods.rs` / `impl_propose.rs` / `impl_verify.rs` — split `EvmApplication` method lanes.
  - `tests/mod.rs` + `tests/*.rs` — directory-backed executor unit tests split by topic with shared fixtures in the parent module.
  - `tests/lifecycle.rs` — source-adjacent `EvmApplication` lifecycle / propose / verify / state-root coverage migrated from the former crate-local `tests/*.rs` tree.
  - `tests/tx_pool.rs` — source-adjacent tx-pool behavior coverage migrated from the former crate-local `tests/*.rs` tree.
  - `mod.rs` now uses explicit module wiring/imports (no `include!` composition for helper files).
  - executor helper functions now avoid scoped visibility modifiers (`pub(super)`/`pub(crate)`), using `pub` inside private modules for parent-module access.
  - production import rewrites now avoid `super::` in the split `config/*`, `impl_*`, decode, and fee-accounting files.
- `full_dkg_strict_height` defaults to `0` (strict from genesis) and can be explicitly overridden via config builders for migration/testing scenarios.
- `WhirlpoolEvmConfig` is now split across directory-backed `config/` submodules so builder/accessor APIs stay grouped by concern while preserving the same external type and behavior.
- `executor/header_and_decode/` and `executor/state_helpers/` are directory-backed modules that split decode/header and state-helper surfaces into focused files with smaller public API sets.
- Non-boundary FullDkg candidate validation is fail-closed in both propose and verify paths: candidate `output.players` must match activation-resolved players for the candidate epoch before include/omit decisions.
- FullDkg inclusion trigger compares candidate output against the **latest committed FullDkg in block storage** (backward scan) so raw-only intermediate blocks do not cause include/omit oscillation.
- When `full_dkg_feature_enabled == false`, verify now rejects **both** `full_dkg` and `reshare` sections instead of rejecting `reshare` alone, preventing disabled-feature metadata from entering historical scans.
- Epoch-boundary helper ownership now lives in `evm_precompiles::epoch`; `app-evm-execution` no longer has a dedicated `epoch_boundary/` module tree.
- `app-evm-execution` keeps a **tiny pipeline call site** for epoch boundaries inside `executor/`:
  - propose/verify load boundary state through `evm_precompiles::load_epoch_boundary_state(...)`
  - propose/verify trigger `evm_precompiles::execute_epoch_boundary_system_call_if_required(...)`
  - canonical epoch writes replay through `evm_precompiles::apply_epoch_boundary_effect(...)`
  - reserved namespace detection is the only epoch adapter left locally, as a small executor helper over `reserved_advance_epoch_call_matches(...)`
- Ownership split for epoch boundaries:
  - `evm-precompiles` owns the pure boundary core (`EpochBoundaryState`, predicate, reserved matcher, typed `EpochBoundaryEffect`) **and** the runtime adapter layer (`StateDb` load/apply + generic system-call support + `EpochBoundaryRuntimeError`).
  - `app-evm-execution` owns pipeline timing and error translation only: propose maps runtime boundary failures to `Execution`, verify maps them to `InvalidBlock`, while shared state-access failures still surface as `State`.
  - the critical sequencing invariant remains unchanged: `apply_pre_execution_changes()` -> boundary system call -> immediate in-memory `commit(outcome.state.clone())` -> user tx execution -> bundle commit -> canonical epoch effect apply -> post-block accounting / extra-data consumers.
- Fee/community-pool ownership split now mirrors the epoch boundary pattern:
  - `evm-precompiles` owns the new internal post-block accounting boundary (`PostBlockAccountingInputs`, `PostBlockAccountingEffect`, `PostBlockAccountingOutcome`, `apply_post_block_accounting`, `PostBlockAccountingRuntimeError`) and the fee/community-pool write logic previously housed in local executor helpers.
  - `app-evm-execution` keeps only executor-native input derivation (`aggregate_priority_fees`) plus propose/verify ordering and runtime-error translation.
  - propose/verify both call the same lower-layer accounting entrypoint after bundle commit and epoch effect application.
- Validator activation semantics are no longer app-owned: `validator_activation/` was removed. Propose/verify call `evm_precompiles::epoch::EpochActivationTargets` for boundary target facts and `evm_precompiles::validators::ValidatorActivationSchedule` for player resolution, mapping lower-layer errors into `EvmAppError::InvalidBlock`.
- Boundary unlock flow:
  - after a successful boundary `advanceEpoch()` call, runtime may unlock community-pool funds
  - cadence is keyed to post-boundary `currentEpoch`
  - unlock cadence regression coverage includes `unlock_every_epochs > 1` with explicit non-multiple-epoch skip + matching-epoch single-application assertions
  - tranche moves from `COMMUNITY_POOL_ADDRESS` -> `FEE_POOL_PRECOMPILE_ADDRESS`
  - tranche is credited into existing fee-pool claim slots by ordered `simplex_validators` addresses with top-k remainder assignment
  - unlock progress is tracked by `lockedRemaining` + `lastProcessedEpoch` slots at the community-pool account
- Fee routing behavior:
  - burned base fees are credited to `evm_precompiles::COMMUNITY_POOL_ADDRESS`
  - regression coverage asserts lower-layer burned-fee account rewrites preserve all community-pool unlock slots (`unlockEveryEpochs`, `unlockAmountPerCycle`, `lockedRemaining`, `lastProcessedEpoch`) on non-boundary blocks
  - priority fees are credited to `evm_precompiles::FEE_POOL_PRECOMPILE_ADDRESS`
  - per-recipient claimable balances are stored in fee-pool precompile storage (`claimable_balance_slot`)
  - proposers withdraw later via fee-pool precompile `withdraw()`
- `suggested_fee_recipient` in execution env is now forced to fee-pool address; block header `proposer_fee_recipient` remains proposer metadata.
- `state::StateDb` is now the only state trait used by `app-evm-execution`; the old subset seam `app_evm_execution::traits::StateProvider` is removed.
- Block gas accounting now uses the final cumulative receipt gas (last receipt), avoiding sum-of-cumulative overcounting.
- On boundary heights, propose executes `advanceEpoch` as an internal system call before user tx execution; no synthetic boundary tx bytes are added to `block.transactions`.
- Reserved epoch namespace tx bytes in the user payload are treated as invalid protocol artifacts: propose excludes them and verify rejects blocks that contain them.
- Proposal and verify now share one executor-local invalid-tx classification seam: `InvalidTx` validation failures are soft-rejected during proposal and reported as `InvalidBlock` during verify, while non-validation execution failures remain `Execution`.
- Reserved-namespace filtering stays behaviorally strict on the app path: only `(system sender, epoch precompile, zero value, advanceEpoch calldata)` is treated as reserved; non-zero value near-miss epoch calls are not filtered as reserved and remain ordinary user transactions.
- `verify()` computes against a cloned state snapshot and validates roots; it does not persist the computed post-state back into `state_db`.
- Proposal cache reuse is now keyed by `(height, parent_id)` (not height alone), and `verify()` fail-closes when `block.parent_id != parent.compute_id()`.
- Finalization receipts are identity-bound to blocks (`height`, `parent_id`, `block_id`) and are cleared only after successful `store_block`; failed persistence retains staged receipts for retry/inspection.
- Finalization allows an explicit no-staged-receipts fallback only for empty-transaction finalized blocks (persists with empty receipts).
- Reth v2 / REVM 36 alignment:
  - execution state uses `revm::database::State` builder with `with_bundle_update()` (no legacy `without_state_clear()` hook),
  - receipt handling/Trie-root encoding is explicitly typed against `reth_ethereum_primitives::Receipt` in propose/verify flows.
- Clippy hygiene:
  - duplicate-proposal cache tuple shape is factored behind a `ProposedCacheEntry` alias,
  - test helpers avoid post-`Default` field reassignment for `AccountInfo`,
  - one-off single-element slices use `std::slice::from_ref(...)`.

## Canonical Imports
- `app_evm_execution::traits::StateDb`
- `app_evm_execution::WhirlpoolEvmConfig`
- `app_evm_execution::EvmApplication`
- `app_evm_execution::decode_evm_transaction`
- `app_evm_execution::decode_evm_transactions`
- `app_evm_execution::ProposedEvmPayload`
- `app_evm_execution::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm_execution::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `chainspec::build_sahara_chain_spec*`
- `chainspec::try_build_sahara_chain_spec*`
- `chainspec::SAHARA_CHAIN_ID`
- `chainspec::try_simplex_validators_from_chain_spec`
