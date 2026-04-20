# app-evm

## Purpose
Pure EVM runtime/config/execution crate for Whirlpool.

## Location
`crates/app/execute/evm/app/`

## Ownership Boundary
`app-evm` now owns EVM behavior, not Sahara chain-spec construction.

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
- `WhirlpoolEvmConfig` also supports epoch-scoped activation overrides via `with_activation_players_for_epoch(epoch, players)`:
  - default behavior (no overrides configured): any queried epoch resolves to `simplex_consensus_public_keys()`
  - when overrides are configured: lookups are strict, and missing target epochs resolve as `None`
- Precompile injection remains in `WhirlpoolEvmConfig::evm_with_env(...)` via `evm_precompiles::whirlpool_precompiles_with_validators(...)`.
- Block header `extra_data` now uses app-shared canonical envelope bytes (`RawEth` + optional `FullDkgV1`) instead of raw proposer key bytes.
- Verify path decodes `extra_data` with a height gate (`Legacy` before strict height, `Strict` at/after strict height), enforces proposer-key parity against `RawEth`, and enforces boundary-aware FullDkg/Reshare invariants.
- Boundary extra-data semantics:
  - when a FullDkg candidate is configured, boundary blocks emit `FullDkgV1` at `E+1` and `ReshareV1(target_epoch=E+2)` where `E` is post-`advanceEpoch` epoch.
  - non-boundary blocks must not carry `ReshareV1`.
  - boundary verify is fail-closed for missing/mismatched required `ReshareV1` fields when FullDkg candidate data is configured.
- Canonical extra-data composition and include/omit predicates are centralized in `canonical_extra_data.rs` (`build_canonical_extra_data`, `full_dkg_should_be_included`, activation-parity guard) and reused by propose/verify paths through `executor/mod.rs`.
- Executor layout is now directory-backed under `app/src/executor/`:
  - `mod.rs` — public façade + `Application` trait implementation wiring.
  - `header_and_decode.rs` — header projection + tx decode helpers.
  - `state_helpers.rs` — internal fee/community-pool/state helper logic.
  - `impl_core_methods.rs` / `impl_propose.rs` / `impl_verify.rs` — split `EvmApplication` method lanes.
  - `tests.rs` — extracted executor unit tests.
  - `mod.rs` now uses explicit module wiring/imports (no `include!` composition for helper files).
  - executor helper functions now avoid scoped visibility modifiers (`pub(super)`/`pub(crate)`), using `pub` inside private modules for parent-module access.
- `full_dkg_strict_height` defaults to `0` (strict from genesis) and can be explicitly overridden via config builders for migration/testing scenarios.
- Non-boundary FullDkg candidate validation is fail-closed in both propose and verify paths: candidate `output.players` must match activation-resolved players for the candidate epoch before include/omit decisions.
- FullDkg inclusion trigger compares candidate output against the **latest committed FullDkg in block storage** (backward scan) so raw-only intermediate blocks do not cause include/omit oscillation.
- Epoch-boundary deterministic system-call handling now lives in `epoch_boundary.rs` and is shared between propose/verify paths.
- Boundary epoch math and activation-derived player resolution are shared through `validator_activation.rs` (`BoundaryEpochContext`, `ActivationSourceResolver`) so propose/verify evaluate the same forward epoch targets.
- `ActivationSourceResolver` is fail-closed for boundary FullDkg/Reshare targeting: a missing configured player set for the required epoch returns `InvalidBlock("activation resolver missing player set for epoch <n>")`.
- Boundary unlock flow:
  - after a successful boundary `advanceEpoch()` call, runtime may unlock community-pool funds
  - cadence is keyed to post-boundary `currentEpoch`
  - unlock cadence regression coverage includes `unlock_every_epochs > 1` with explicit non-multiple-epoch skip + matching-epoch single-application assertions
  - tranche moves from `COMMUNITY_POOL_ADDRESS` -> `FEE_POOL_PRECOMPILE_ADDRESS`
  - tranche is credited into existing fee-pool claim slots by ordered `simplex_validators` addresses with top-k remainder assignment
  - unlock progress is tracked by `lockedRemaining` + `lastProcessedEpoch` slots at the community-pool account
- Fee routing behavior:
  - burned base fees are credited to `evm_precompiles::COMMUNITY_POOL_ADDRESS`
  - regression coverage asserts burned-fee account rewrites preserve all community-pool unlock slots (`unlockEveryEpochs`, `unlockAmountPerCycle`, `lockedRemaining`, `lastProcessedEpoch`) on non-boundary blocks
  - priority fees are credited to `evm_precompiles::FEE_POOL_PRECOMPILE_ADDRESS`
  - per-recipient claimable balances are stored in fee-pool precompile storage (`claimable_balance_slot`)
  - proposers withdraw later via fee-pool precompile `withdraw()`
- `suggested_fee_recipient` in execution env is now forced to fee-pool address; block header `proposer_fee_recipient` remains proposer metadata.
- `state::StateDb` writes are now used for claim-ledger slot updates via `insert_storage`.
- Block gas accounting now uses the final cumulative receipt gas (last receipt), avoiding sum-of-cumulative overcounting.
- On boundary heights, propose executes `advanceEpoch` as an internal system call before user tx execution; no synthetic boundary tx bytes are added to `block.transactions`.
- Reserved epoch namespace tx bytes in the user payload are treated as invalid protocol artifacts: propose excludes them and verify rejects blocks that contain them.
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
- `app_evm::traits::StateProvider`
- `app_evm::WhirlpoolEvmConfig`
- `app_evm::EvmApplication`
- `app_evm::decode_evm_transaction`
- `app_evm::decode_evm_transactions`
- `app_evm::ProposedEvmPayload`
- `app_evm::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `chainspec::build_sahara_chain_spec*`
- `chainspec::try_build_sahara_chain_spec*`
- `chainspec::SAHARA_CHAIN_ID`
- `chainspec::try_simplex_validators_from_chain_spec`
