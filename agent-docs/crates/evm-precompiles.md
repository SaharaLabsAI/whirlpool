# evm-precompiles

## Purpose
Workspace-owned registry and implementation crate for Whirlpool custom EVM precompiles.

## Location
`crates/app/execute/evm/precompiles/`

## Key exports
- `WhirlpoolEvmFactory`: custom EVM factory that injects Whirlpool precompiles into `EthEvmBuilder`.
- `whirlpool_precompiles(spec) -> PrecompilesMap`: builds builtin+Whirlpool precompile map for a given spec.
- `whirlpool_precompiles_with_validators(spec, validators) -> PrecompilesMap`: builds builtin+Whirlpool precompile map with a captured ordered simplex-validator list.
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
- `EPOCH_PRECOMPILE_ADDRESS`: stateful epoch metadata precompile endpoint.
- `current_epoch_calldata()`
- `next_epoch_block_calldata()`
- `epoch_blocks_calldata()`
- `epoch_start_block_calldata(epoch)`
- `advance_epoch_calldata()`
- `epoch_system_tx_sender()`
- `is_advance_epoch_calldata(bytes)`
- epoch storage slot helpers:
  - `current_epoch_slot()`, `epoch_blocks_slot()`, `next_epoch_block_slot()`
  - `epoch_start_block_slot(epoch)` + `encode_epoch_start_block_storage_value(...)`

## Framework shape
- `src/lib.rs`: registry, duplicate-address protection, safe-default stateful registration guard, factory wiring, crate-level tests.
- `src/community_pool/mod.rs`: canonical community-pool address constant + read-only balance query precompile and ABI helpers.
- `src/fee_pool/mod.rs`: fee-pool precompile surface, ABI helpers, revert helpers, and tests.
- `src/fee_pool/dispatch.rs`: alloy `sol!` selectors for `feePoolBalance()`, `claimableBalance(address)`, and `withdraw()`.
- `src/fee_pool/impl.rs`: stateful fee-pool logic (balance query, claim query, withdraw transfer, claim reset).
- `src/fee_pool/storage.rs`: deterministic slot derivation for `mapping(address => uint256) claimable`.
- `src/fee_pool/gas.rs`: fee-pool gas schedule.
- `src/validators/mod.rs`: ordered simplex-validator precompile ABI, output encoder/decoder, and tests.
- `src/epoch/mod.rs`: epoch constants, sender derivation, ABI helper exports, and epoch tests.
- `src/epoch/dispatch.rs`: alloy `sol!` selectors for epoch read/write ABI.
- `src/epoch/impl.rs`: stateful epoch logic with restricted `advanceEpoch()`.
- `src/epoch/storage.rs`: scalar slots + append-only epoch-start mapping slot derivation.
- `src/epoch/gas.rs`: epoch precompile gas schedule.

## Design notes
- Reth v2 alignment: this crate uses `reth_evm::revm::*` types everywhere (no direct `revm` crate import) to avoid mixed-REVM type graphs during factory wiring.
- Custom precompiles are installed through `PrecompilesMap` dynamic entries, not vendor edits.
- The validators precompile is read-only and returns the ordered list provided by the canonical Rust validator reader (`validators` crate).
- The community-pool precompile is read-only and returns the balance of `COMMUNITY_POOL_ADDRESS`.
- Community-pool unlock schedule state is also anchored at `COMMUNITY_POOL_ADDRESS` storage (configured by chainspec/runtime), but unlock execution itself is runtime logic in `app-evm`, not a mutable community-pool precompile method.
- Fee routing model:
  - burned base fees -> `COMMUNITY_POOL_ADDRESS`
  - priority fees -> `FEE_POOL_PRECOMPILE_ADDRESS`
  - proposer entitlement -> fee-pool claim ledger keyed by recipient address
  - payout -> precompile `withdraw()` path
- The fee-pool precompile mutates journaled EVM **account balances** and claim-ledger storage through the shared EVM internals.
- Epoch precompile model:
  - read selectors: `currentEpoch`, `nextEpochBlock`, `epochBlocks`, `epochStartBlock(epoch)`
  - write selector: `advanceEpoch()` (restricted to `epoch_system_tx_sender()`, non-static)
  - append-only epoch start map uses plus-one storage encoding so epoch 0 start block can be stored unambiguously.
- Whirlpool-owned stateful precompiles registered via `RegisteredPrecompile::new_stateful` are direct-call-only: the final hop must keep `target_address == bytecode_address`, which allows ordinary `CALL`/`STATICCALL` and rejects delegate-style execution.
- Non-direct-call rejection is a framework-level revert emitted before the target handler runs, so it reports zero precompile-local `gas_used`; enclosing EVM call/setup overhead is still charged outside the precompile.
- Top-level EOAs calling a precompile address directly are not the only validated path here; the full-node tests use a tiny forwarding contract that performs an internal ordinary `CALL` into the precompile, which remains valid because the precompile boundary is still direct.

## Verification
- Crate tests cover registry construction, duplicate-address rejection, dispatch routing, direct-call boundary enforcement, gas behavior, revert mapping, and fee-pool withdraw/claim invariants.
