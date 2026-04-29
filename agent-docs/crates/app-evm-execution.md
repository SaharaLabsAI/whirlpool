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
- Fee-recipient block timing and lower-layer error mapping

### Does not own anymore
- `SAHARA_CHAIN_ID`
- `build_sahara_chain_spec*`
- `try_build_sahara_chain_spec*`
- public `try_simplex_validators_from_chain_spec`

Those live in `chainspec`.

## Key Runtime Notes
- Proposer fee-recipient authority now comes from runtime validator state via `evm_precompiles::validators`, not from app config or a separate fee-recipient registry. Propose/verify resolve the proposer against the parent/pre-block active registry and fail closed on missing proposer, malformed registry, or carried-recipient mismatch.
- Post-block accounting loads validator ordering from the post-execution/pre-accounting runtime registry snapshot, so fee/unlock distribution follows the same validator state being finalized.
- `WhirlpoolEvmConfig` carries ordered genesis validator-registry entries for EVM factory wiring and DKG default-player inputs. Runtime fee recipient and accounting validator ordering are loaded from state through `evm-precompiles`.
- `WhirlpoolEvmConfig` carries FullDkg envelope knobs:
  - `full_dkg_feature_enabled`
  - optional `current_full_dkg_output` (`dealers`, `players`, `public_polynomial`)
- `WhirlpoolEvmConfig` carries epoch-scoped activation override data via `with_activation_players_for_epoch(epoch, players)`, but activation resolution is delegated to `validators_dkg::ValidatorActivationSchedule`.
- Precompile injection remains in `WhirlpoolEvmConfig::evm_with_env(...)` via `evm_precompiles::whirlpool_precompiles_with_validators(...)`.
- Block header `extra_data` uses strict canonical envelope bytes (`RawEth` + optional `FullDkgV1`/`ReshareV1`). `app-primitives` owns carrier helper wrappers; `validators-dkg` owns DKG schema/validation semantics.
- Verify path decodes `extra_data` strictly from genesis, rejects raw 32-byte legacy carriers, enforces proposer-key parity against `RawEth`, and enforces boundary-aware FullDkg/Reshare invariants.
- Propose/verify now share one block-pipeline-local next-block base-fee seam:
  - propose derives `base_fee_per_gas` through the shared helper,
  - verify rejects blocks whose `block.base_fee_per_gas` does not match the protocol-derived next-block fee before fee accounting,
  - verify-side burned-fee / priority-fee accounting now uses the derived canonical fee after the mismatch guard.
- EVM tx decode helpers now use exact EIP-2718 decoding, so padded tx bytes fail closed during both proposal pre-decode and verify decoding.
- Boundary extra-data semantics are delegated to `validators-dkg`: `EpochActivationTargets` supplies `E`, `E+1`, and `E+2`; `ValidatorActivationSchedule` resolves FullDkg/Reshare players. Non-boundary blocks must not carry `ReshareV1`, and boundary verify remains fail-closed for missing/mismatched required `ReshareV1` fields when FullDkg candidate data is configured.
- `app-evm-execution` is only the DKG pipeline call site: propose calls `validators_dkg::latest_committed_full_dkg` and `build_canonical_dkg_extra_data`; verify calls `latest_committed_full_dkg` and `validate_dkg_extra_data`. Strict header carrier decode/proposer extraction routes through `app_primitives::header_extra_data`. Historical carrier-byte lookup is supplied by the DB through `validators_dkg::DkgHistory`; execution keeps only DKG error translation.
- Reviewer entrypoints are now named by pipeline ownership zone:
  - `src/ingress.rs` — candidate transaction sources: proposal reads `TxSource::pending()`, verification borrows `block.transactions`.
  - `src/codec/` — EIP-2718 transaction decode/recovery plus EVM header projection. Prefer `app_evm_execution::codec::decode_evm_transaction` and `decode_evm_transactions` for reviewer-facing decode APIs; root re-exports remain for compatibility.
  - `src/block_pipeline/` — `EvmApplication`, `Application` trait wiring, explicit `propose.rs` and `verify.rs` lanes, plus private `accounting/` helpers for fee and receipt derivation.
  - `src/post_handle.rs` — `ReceiptStore` owns staged/pending receipt state, finalization persistence, and `pending_receipts` visibility.
  - `block_pipeline/tests/mod.rs` + `block_pipeline/tests/*.rs` — source-adjacent unit tests split by topic with shared fixtures in the parent module.
- The former `app_evm_execution::executor::*` compatibility shim was intentionally removed after the ownership-zone modules became the public review map; use root exports or `codec`/`block_pipeline` paths instead.
- The discoverability refactor is behavior-preserving: no shared generic propose/verify pipeline abstraction was added, and deeper ownership issues should be tracked as remaining risks rather than repaired in this layout pass.
- `WhirlpoolEvmConfig` is now split across directory-backed `config/` submodules so builder/accessor APIs stay grouped by concern while preserving the same external type and behavior. Validator-registry snapshot access lives in `config/validator_registry.rs`; DKG call-site config lives under `config/dkg/`.
- `codec/` remains directory-backed for decode/header surfaces; `block_pipeline/accounting/` groups fee and receipt derivation as a concern-specific private module rather than a generic helper bucket.
- Non-boundary FullDkg candidate validation is fail-closed in both propose and verify paths: candidate `output.players` must match activation-resolved players for the candidate epoch before include/omit decisions.
- FullDkg inclusion trigger compares candidate output against the **latest committed FullDkg from `DkgHistory`** (validators-dkg backward scan over raw carrier bytes) so raw-only intermediate blocks do not cause include/omit oscillation.
- When `full_dkg_feature_enabled == false`, propose still emits a canonical RawEth envelope, and verify rejects **both** `full_dkg` and `reshare` sections, preventing disabled-feature metadata from entering historical scans.
- Epoch-boundary helper ownership now lives in `evm_precompiles::epoch`; `app-evm-execution` no longer has a dedicated `epoch_boundary/` module tree.
- `app-evm-execution` keeps a **tiny pipeline call site** for epoch boundaries inside `block_pipeline/`:
  - propose/verify load boundary state through `evm_precompiles::load_epoch_boundary_state(...)`
  - propose/verify trigger `evm_precompiles::execute_epoch_boundary_system_call_if_required(...)`
  - canonical epoch writes replay through `evm_precompiles::apply_epoch_boundary_effect(...)`
  - reserved namespace detection is the only epoch adapter left locally, as a small block-pipeline helper over `reserved_advance_epoch_call_matches(...)`
- Ownership split for epoch boundaries:
  - `evm-precompiles` owns the pure boundary core (`EpochBoundaryState`, predicate, reserved matcher, typed `EpochBoundaryEffect`) **and** the runtime adapter layer (`StateDb` load/apply + generic system-call support + `EpochBoundaryRuntimeError`).
  - `app-evm-execution` owns pipeline timing and error translation only: propose maps runtime boundary failures to `Execution`, verify maps them to `InvalidBlock`, while shared state-access failures still surface as `State`.
  - the critical sequencing invariant remains unchanged: `apply_pre_execution_changes()` -> boundary system call -> immediate in-memory `commit(outcome.state.clone())` -> user tx execution -> bundle commit -> canonical epoch effect apply -> post-block accounting / extra-data consumers.
- Fee/community-pool ownership split now mirrors the epoch boundary pattern:
  - `evm-precompiles` owns the new internal post-block accounting boundary (`PostBlockAccountingInputs`, `PostBlockAccountingEffect`, `PostBlockAccountingOutcome`, `apply_post_block_accounting`, `PostBlockAccountingRuntimeError`) and the fee/community-pool write logic previously housed in local pipeline helpers.
  - `app-evm-execution` keeps only pipeline-native input derivation (`aggregate_priority_fees`) plus propose/verify ordering and runtime-error translation.
  - propose/verify both call the same lower-layer accounting entrypoint after bundle commit and epoch effect application.
- Validator activation and DKG metadata semantics are no longer execution-owned. Propose/verify call `validators-dkg` and only map DKG errors into `EvmAppError::InvalidBlock`.
- Boundary unlock flow:
  - after a successful boundary `advanceEpoch()` call, runtime may unlock community-pool funds
  - cadence is keyed to post-boundary `currentEpoch`
  - unlock cadence regression coverage includes `unlock_every_epochs > 1` with explicit non-multiple-epoch skip + matching-epoch single-application assertions
  - tranche moves from `COMMUNITY_POOL_ADDRESS` -> `FEE_POOL_PRECOMPILE_ADDRESS`
  - tranche is credited into existing fee-pool claim slots by ordered validator-registry addresses with top-k remainder assignment
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
- Proposal and verify now share one block-pipeline-local invalid-tx classification seam: `InvalidTx` validation failures are soft-rejected during proposal and reported as `InvalidBlock` during verify, while non-validation execution failures remain `Execution`.
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
- `app_evm_execution::codec::decode_evm_transaction`
- `app_evm_execution::codec::decode_evm_transactions`
- `app_evm_execution::decode_evm_transaction` (compatibility root re-export)
- `app_evm_execution::decode_evm_transactions` (compatibility root re-export)
- `app_evm_execution::ProposedEvmPayload`
- `chainspec::build_sahara_chain_spec*`
- `chainspec::try_build_sahara_chain_spec*`
- `chainspec::SAHARA_CHAIN_ID`
- `chainspec::try_simplex_validators_from_chain_spec`
