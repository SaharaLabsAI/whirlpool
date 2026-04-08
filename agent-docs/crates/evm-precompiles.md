# evm-precompiles

## Purpose
Workspace-owned registry and implementation crate for Whirlpool custom EVM precompiles.

## Location
`crates/evm/precompiles/`

## Key exports
- `WhirlpoolEvmFactory`: custom EVM factory that injects Whirlpool precompiles into `EthEvmBuilder`.
- `whirlpool_precompiles(spec) -> PrecompilesMap`: builds builtin+Whirlpool precompile map for a given spec.
- `NonDirectCall`: shared ABI-visible framework error for non-direct Whirlpool precompile execution.
- `TEST_TOKEN_PRECOMPILE_ADDRESS`
- `mint_calldata(address, amount)`
- `balance_of_calldata(address)`

## Framework shape
- `src/lib.rs`: registry, duplicate-address protection, safe-default stateful registration guard, factory wiring, crate-level tests.
- `src/test_token/mod.rs`: public surface + error/revert helpers.
- `src/test_token/dispatch.rs`: alloy `sol!` ABI definitions plus calldata decode/encode helpers.
- `src/test_token/gas.rs`: example per-precompile gas policy.
- `src/test_token/impl.rs`: example stateful business logic.

## Design notes
- Custom precompiles are installed through `PrecompilesMap` dynamic entries, not vendor edits.
- The example `test-token` precompile is validation scaffolding, not a default product feature.
- The current example mutates journaled EVM **account balances** and exposes them through a precompile ABI.
- Whirlpool-owned stateful precompiles registered via `RegisteredPrecompile::new_stateful` are direct-call-only: the final hop must keep `target_address == bytecode_address`, which allows ordinary `CALL`/`STATICCALL` and rejects delegate-style execution.
- Top-level EOAs calling a precompile address directly are not the only validated path here; the full-node tests use a tiny forwarding contract that performs an internal ordinary `CALL` into the precompile, which remains valid because the precompile boundary is still direct.

## Verification
- Crate tests cover registry construction, duplicate-address rejection, dispatch routing, direct-call boundary enforcement, gas behavior, and revert mapping.
