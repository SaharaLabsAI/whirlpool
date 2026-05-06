# evm-precompiles

## Purpose
Workspace-owned registry and implementation crate for Whirlpool custom EVM precompiles.

## Location
`crates/precompiles/evm/`

## Key exports
- `WhirlpoolEvmFactory`: custom EVM factory that injects Whirlpool precompiles into `EthEvmBuilder`; `with_validators(...)` is a compatibility constructor for existing validator-aware call sites, but validator data is read from runtime EVM state.
- `whirlpool_precompiles(spec) -> PrecompilesMap`: compatibility helper for a zero-validator bootstrap/test registry; not the canonical runtime path.
- `whirlpool_precompiles_with_validators(spec, validators) -> PrecompilesMap`: compatibility constructor for builtin+Whirlpool precompile maps; validators runtime reads now come from EVM state.
- `build_whirlpool_precompiles(spec) -> Result<PrecompilesMap, RegistryError>`: compatibility helper for a zero-validator bootstrap/test registry; not the canonical runtime path.
- `build_whirlpool_precompiles_with_validators(spec, validators) -> Result<PrecompilesMap, RegistryError>`: compatibility builder for validator-aware call sites; validator data is read from runtime EVM state.
- `NonDirectCall`: shared ABI-visible framework error for non-direct Whirlpool precompile execution.
- `COMMUNITY_POOL_ADDRESS`: canonical single-address business sink and read-only precompile endpoint for community-pool balance.
- `community_pool_balance_calldata()`
- `decode_community_pool_balance_output(bytes)`
- community-pool unlock storage helpers:
  - `community_pool_unlock_every_epochs_{storage_slot,slot}()`
  - `community_pool_unlock_amount_per_cycle_{storage_slot,slot}()`
  - `community_pool_locked_remaining_{storage_slot,slot}()`
  - `community_pool_last_processed_epoch_{storage_slot,slot}()`
  - `encode_u256_storage_value(...)`
- `FEE_POOL_PRECOMPILE_ADDRESS`: stateful fee-pool precompile endpoint for priority-fee sink + claim ledger.
- `fee_pool_balance_calldata()`
- `claimable_balance_calldata(address)`
- `withdraw_calldata()`
- `claimable_balance_slot(address)`
- `decode_fee_pool_balance_output(bytes)`
- `decode_claimable_balance_output(bytes)`
- `decode_withdraw_output(bytes)`
- `VALIDATORS_PRECOMPILE_ADDRESS`
- `validators_calldata()`
- `decode_validators_output(bytes)`
- validators precompile ABI helpers: `VALIDATORS_PRECOMPILE_ADDRESS`, `validators_calldata()`, `decode_validators_output(bytes)`; canonical registry reader types/codecs live in `validators-reader`.
- validators runtime-state helpers: `load_active_validator_registry`, `resolve_active_validator_fee_recipient`, `validate_active_validator_fee_recipient`, `ValidatorsRuntimeError`.
- `EPOCH_PRECOMPILE_ADDRESS`: stateful epoch metadata precompile endpoint.
- `current_epoch_calldata()`
- `next_epoch_block_calldata()`
- `epoch_blocks_calldata()`
- `epoch_start_block_calldata(epoch)`
- `advance_epoch_calldata()`
- `epoch_system_tx_sender()`
- `is_advance_epoch_calldata(bytes)`
- `EpochBoundaryState { next_epoch_block }`
- `EpochBoundaryEffect { writes: [EpochBoundaryStorageWrite; 3] }`
- `EpochBoundaryStorageWrite { slot, value }`
- `EpochBoundaryEffectError`
- `extract_epoch_boundary_effect(outcome_state)`
- `boundary_required_for_height(state, block_height)`
- `reserved_advance_epoch_call_matches(caller, target_address, value, calldata)`
- epoch storage slot helpers:
  - `current_epoch_slot()`, `epoch_blocks_slot()`, `next_epoch_block_slot()`
  - `epoch_start_block_slot(epoch)` + `encode_epoch_start_block_storage_value(...)`

## Framework shape
- `src/lib.rs`: thin crate facade with public compatibility re-exports only.
- `src/factory/mod.rs`: `WhirlpoolEvmFactory`, `impl EvmFactory`, and `with_validators(...)` compatibility constructor.
- `src/registry/{entry,direct_call_guard,installed,build,runtime}.rs`: registered-precompile metadata, direct-call rejection, canonical installed list, checked map construction, and panic-on-invalid compatibility helpers.
- `src/tests.rs`: crate-root registry/factory/direct-call tests.
- `src/community_pool/mod.rs` + `community_pool/tests.rs`: canonical community-pool address, read-only balance precompile shell, ABI helpers, and facade tests.
- `src/community_pool/unlock_accounting/{transition,runtime}.rs`: pure post-block unlock/fee-accounting effect planning plus runtime apply adapter; tests stay source-adjacent as `*_tests.rs` files.
- `src/community_pool/unlock_storage/{storage_slots,storage_encoding,value_slots,last_processed}.rs`: community-pool unlock slot/value helper ownership.
- `src/fee_pool/mod.rs` + `fee_pool/tests.rs`: fee-pool facade, ABI compatibility exports including the legacy `fee_pool::storage::claimable_balance_slot` path, registration handoff, revert helpers, and runtime tests.
- `src/fee_pool/codec/{dispatch,calldata,output}.rs`: fee-pool selector decode, calldata helpers, and ABI output decode helpers.
- `src/fee_pool/transition/withdraw/mod.rs`: pure typed withdraw planner (`WithdrawInput`, `WithdrawState`, `WithdrawEffect`, `WithdrawOutcome`) and source-adjacent transition tests.
- `src/fee_pool/runtime/{handler,state,effect_writer}.rs`: stateful precompile shell, runtime snapshot loading, and canonical withdraw effect writer.
- `src/fee_pool/claim_ledger/{slots,credit,runtime_writer}.rs`: claimable-balance slot owner, `ClaimCredit` carrier, and shared runtime claim writer used by withdraw/post-block accounting.
- `src/epoch/mod.rs` + `epoch/tests.rs`: epoch facade, public exports, constants, and ABI/runtime tests.
- `src/epoch/codec/{dispatch,read_calldata,write_calldata,output_scalar,output_epoch_start}.rs`: epoch selector decode, calldata helpers, and output decoding.
- `src/epoch/transition/{advance,boundary_effect,boundary_semantics}.rs`: pure direct-advance planner, boundary effect extraction, and primitive boundary predicates.
- `src/epoch/runtime/{handler,state,effect_writer,boundary_adapter}.rs`: direct `advanceEpoch()` shell, storage snapshot loading, canonical direct-write apply path, and system-call boundary adapter.
- `src/epoch/storage/{codec,encoded_value,epoch_start_mapping,well_known_slots,well_known_storage}.rs`: scalar slots, epoch-start mapping, plus-one storage encoding, and storage-ready slot conversions.
- `src/epoch/registration/mod.rs`: epoch precompile registration and deterministic system transaction sender.
- `src/validators/mod.rs` + `validators/tests.rs`: validators read-only precompile ABI/runtime facade.
- `src/validators/runtime_state/{mod,precompile_adapter,registry_loader}.rs`: active validator runtime authority API, precompile input adapter, and shared slot-walking loader; source-adjacent malformed-registry tests stay in `runtime_state/tests.rs`.
- `src/validators/gas.rs`: standalone validators gas schedule.

## Design notes
- Reth v2 alignment: this crate uses `reth_evm::revm::*` types everywhere (no direct `revm` crate import) to avoid mixed-REVM type graphs during factory wiring.
- Custom precompiles are installed through `PrecompilesMap` dynamic entries, not vendor edits.
- Canonical runtime wiring still exposes validator-aware constructors for compatibility, but `validators()` now reads `SIMPLEX_VALIDATORS_REGISTRY` from runtime EVM state rather than a captured constructor snapshot.
- `validators-reader` is the canonical owner of validator registry codec/types and slot arithmetic. `validators-dkg` owns activation schedules/targets and DKG metadata. `evm-precompiles::validators` owns runtime validator-state reads, active-registry loading, proposer fee-recipient resolution, malformed-registry classification, and the public validators ABI.
- Current active-set semantics: runtime `SIMPLEX_VALIDATORS_REGISTRY` membership is the active proof for fee-recipient resolution. Missing proposer, duplicate pubkey, zero pubkey/address under nonzero count, invalid address padding, and carried-recipient mismatch fail closed.
- The community-pool precompile is read-only and returns the balance of `COMMUNITY_POOL_ADDRESS`.
- Community-pool unlock schedule state remains anchored at `COMMUNITY_POOL_ADDRESS` storage (configured by chainspec/runtime), but the mutable runtime ownership now lives in the internal `unlock_accounting::runtime` adapter surface rather than in `app-evm-execution` or a new public precompile selector.
- Fee routing model:
  - burned base fees -> `COMMUNITY_POOL_ADDRESS`
  - priority fees -> `FEE_POOL_PRECOMPILE_ADDRESS`
  - proposer entitlement -> fee-pool claim ledger keyed by recipient address
  - payout -> precompile `withdraw()` path
- The fee-pool precompile mutates journaled EVM **account balances** and claim-ledger storage through the shared EVM internals. Withdraw transition decisions are planned in the pure `fee_pool::transition::withdraw` module, while `fee_pool::runtime::{handler,effect_writer}` owns the runtime apply/writer shell for returned effects. Post-block accounting credits the same claim ledger through the semantic owner path `fee_pool::claim_ledger::runtime_writer`, avoiding any root-level alias that would obscure claim-ledger ownership.
- The accounting module now owns a second **two-layer boundary API** for post-block accounting:
  - **pure core**: `PostBlockAccountingInputs`, `PostBlockAccountingEffect`, `PostBlockAccountingOutcome`, community-pool unlock state/effect math
  - **runtime adapter**: `apply_post_block_accounting(...)` + `PostBlockAccountingRuntimeError`
- Accounting boundary rule: `app-evm-execution` may compute executor-native inputs (for example aggregated priority fees) and choose when accounting runs, but fee/community-pool slot knowledge, balance-preserving rewrites, unlock share distribution, and claim-ledger mutation now stay inside `evm-precompiles`.
- Epoch precompile model:
  - read selectors: `currentEpoch`, `nextEpochBlock`, `epochBlocks`, `epochStartBlock(epoch)`
  - write selector: `advanceEpoch()` (restricted to `epoch_system_tx_sender()`, non-static)
  - append-only epoch start map uses plus-one storage encoding so epoch 0 start block can be stored unambiguously.
- The epoch module owns pure epoch boundary mechanics consumed by `app-evm-execution`: boundary snapshot type, `block_height == next_epoch_block` predicate, and the canonical reserved-call matcher. DKG activation target handoff (`E`, `E+1`, `E+2`) lives in `validators-dkg`.
- The epoch module also owns two typed epoch-write handoffs: the direct `transition::advance` planner used by the precompile handler, and the boundary-effect extractor used for post-execution canonical application:
  - effect is limited to epoch-precompile storage writes,
  - `epochStartBlock(next_epoch)` stays storage-ready with plus-one encoding,
  - extractor rejects account-info replay requirements and unexpected changed accounts,
  - when REVM omits a dirty `nextEpochBlock` write from the changed-slot set, extraction reconstructs it from loaded runtime context (`old nextEpochBlock + epochBlocks`) instead of deriving it from `epochStartBlock`.
- The epoch module now has a **two-layer boundary API**:
  - **pure core**: primitive/value-only semantics, direct advance planning, and typed effect extraction (`EpochBoundaryState`, predicate, reserved matcher, `AdvanceEpochEffect`, `EpochBoundaryEffect`)
  - **runtime adapter**: `StateDb`-based boundary state load/apply plus generic REVM system-call support (`load_epoch_boundary_state`, `apply_epoch_boundary_effect`, `execute_epoch_boundary_system_call_if_required`, `EpochBoundaryRuntimeError`)
- Runtime adapter fail-closed rule: `execute_epoch_boundary_system_call_if_required` validates/extracts the typed `EpochBoundaryEffect` from system-call `EvmState` before committing that state to the EVM DB; extraction failures return `EffectExtraction` without DB mutation.
- Updated seam rule: the **pure core** remains primitive/value-only and free of app/runtime traits; the **runtime adapter** is intentionally allowed to depend on `state::StateDb` and EVM execution traits, but still does not depend on app-local types such as `TransactionSigned` or `EvmAppError`.
- Whirlpool-owned stateful precompiles registered via `RegisteredPrecompile::new_stateful` are direct-call-only: the final hop must keep `target_address == bytecode_address`, which allows ordinary `CALL`/`STATICCALL` and rejects delegate-style execution.
- Non-direct-call rejection is a framework-level revert emitted before the target handler runs, so it reports zero precompile-local `gas_used`; enclosing EVM call/setup overhead is still charged outside the precompile.
- Top-level EOAs calling a precompile address directly are not the only validated path here; the full-node tests use a tiny forwarding contract that performs an internal ordinary `CALL` into the precompile, which remains valid because the precompile boundary is still direct.

## Verification
- Crate tests are source-adjacent `tests.rs` modules and cover registry construction, duplicate-address rejection, dispatch routing, direct-call boundary enforcement, gas behavior, revert mapping, fee-pool withdraw/claim invariants, pure withdraw and epoch-advance transition planning, epoch semantic-core parity checks, validators runtime-state loading, and runtime-adapter `StateDb` load/apply plus fail-closed validate-before-commit coverage.
- Test helper note: the shared precompile-call helper intentionally keeps a wide argument list and is locally annotated for `clippy::too_many_arguments`.
