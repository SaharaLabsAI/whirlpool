# Precompiles Design

## Summary
Whirlpool custom precompiles are node/runtime features, not contracts that must be deployed. The registry is constructed in `crates/precompiles/evm/src/lib.rs:94-124` and attached to the EVM builder in both `crates/precompiles/evm/src/lib.rs:142-163` and `crates/app/execute/evm/app/src/config.rs:195-199`.

## Why this design exists
- Keep custom behavior in workspace-owned code instead of `vendor/`; see `crates/precompiles/evm/src/lib.rs:94-124` and `agent-docs/crates/evm-precompiles.md`.
- Make precompiles available everywhere the chain executes EVM logic, including proposal, verification, and RPC `eth_call`; see `crates/app/execute/evm/app/src/config.rs:195-199`, `crates/app/execute/evm/app/src/config.rs:257-275`, and `agent-docs/crates/rpc-eth.md`.
- Centralize registration so adding a new Whirlpool precompile means extending one registry instead of editing multiple execution paths; see `crates/precompiles/evm/src/lib.rs:119-124`.
- Put safety policy in the framework layer before business logic runs; see `crates/precompiles/evm/src/lib.rs:47-60` and `crates/precompiles/evm/src/fee_pool/mod.rs`.

## Read next
1. `availability.md` — lifecycle and genesis availability.
2. `call-model.md` — direct-call boundary and state mutation rules.
3. `wiring.md` — registry/factory/config seam and validation coverage.
