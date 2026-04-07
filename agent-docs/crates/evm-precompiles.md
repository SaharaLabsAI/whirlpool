# evm-precompiles

## Purpose
Workspace-owned registry and implementation crate for Whirlpool custom EVM precompiles.

## Location
`crates/evm/precompiles/`

## Key exports
- `WhirlpoolEvmFactory`: custom EVM factory that injects Whirlpool precompiles into `EthEvmBuilder`.
- `whirlpool_precompiles(spec) -> PrecompilesMap`: builds builtin+Whirlpool precompile map for a given spec.
- `TEST_TOKEN_PRECOMPILE_ADDRESS`
- `mint_calldata(address, amount)`
- `balance_of_calldata(address)`

## Framework shape
- `src/lib.rs`: registry, duplicate-address protection, factory wiring, crate-level tests.
- `src/test_token/mod.rs`: public surface + error/revert helpers.
- `src/test_token/dispatch.rs`: selector decoding and calldata encoding.
- `src/test_token/gas.rs`: example per-precompile gas policy.
- `src/test_token/impl.rs`: example stateful business logic.

## Design notes
- Custom precompiles are installed through `PrecompilesMap` dynamic entries, not vendor edits.
- The example `test-token` precompile is validation scaffolding, not a default product feature.
- The current example mutates journaled EVM **account balances** and exposes them through a precompile ABI.
- Top-level EOAs calling a precompile address directly are not the validated path here; the full-node tests use a tiny forwarding contract that performs an internal EVM call into the precompile.

## Verification
- Crate tests cover registry construction, duplicate-address rejection, dispatch routing, gas behavior, and revert mapping.
