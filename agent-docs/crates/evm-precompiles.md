# evm-precompiles

## Purpose
Workspace-owned registry and implementation crate for Whirlpool custom EVM precompiles.

## Location
`crates/evm/precompiles/`

## Key exports
- `WhirlpoolEvmFactory`: custom EVM factory that injects Whirlpool precompiles into `EthEvmBuilder`.
- `whirlpool_precompiles(spec) -> PrecompilesMap`: builds builtin+Whirlpool precompile map for a given spec.
- `whirlpool_precompiles_with_validators(spec, validators) -> PrecompilesMap`: builds builtin+Whirlpool precompile map with a captured ordered simplex-validator list.
- `NonDirectCall`: shared ABI-visible framework error for non-direct Whirlpool precompile execution.
- `COMMUNITY_POOL_ADDRESS`: canonical single-address business sink and read-only precompile endpoint for community-pool balance.
- `community_pool_balance_calldata()`
- `decode_community_pool_balance_output(bytes)`
- `TEST_TOKEN_PRECOMPILE_ADDRESS`
- `VALIDATORS_PRECOMPILE_ADDRESS`
- `mint_calldata(address, amount)`
- `balance_of_calldata(address)`
- `validators_calldata()`
- `decode_validators_output(bytes)`

## Framework shape
- `src/lib.rs`: registry, duplicate-address protection, safe-default stateful registration guard, factory wiring, crate-level tests.
- `src/community_pool/mod.rs`: canonical community-pool address constant + read-only balance query precompile and ABI helpers.
- `src/test_token/mod.rs`: public surface + error/revert helpers.
- `src/test_token/dispatch.rs`: alloy `sol!` ABI definitions plus calldata decode/encode helpers.
- `src/test_token/gas.rs`: example per-precompile gas policy.
- `src/test_token/impl.rs`: example stateful business logic.
- `src/validators/mod.rs`: ordered simplex-validator precompile ABI, output encoder/decoder, and tests.

## Design notes
- Custom precompiles are installed through `PrecompilesMap` dynamic entries, not vendor edits.
- The example `test-token` precompile is validation scaffolding, not a default product feature.
- The validators precompile is read-only and returns the ordered list provided by the canonical Rust validator reader (`validators` crate).
- The community-pool precompile is read-only and returns the balance of `COMMUNITY_POOL_ADDRESS`.
- Single-address model: `COMMUNITY_POOL_ADDRESS` is both the fee-credit sink and the precompile query address.
- The current example mutates journaled EVM **account balances** and exposes them through a precompile ABI.
- Whirlpool-owned stateful precompiles registered via `RegisteredPrecompile::new_stateful` are direct-call-only: the final hop must keep `target_address == bytecode_address`, which allows ordinary `CALL`/`STATICCALL` and rejects delegate-style execution.
- Non-direct-call rejection is a framework-level revert emitted before the target handler runs, so it reports zero precompile-local `gas_used`; enclosing EVM call/setup overhead is still charged outside the precompile.
- Top-level EOAs calling a precompile address directly are not the only validated path here; the full-node tests use a tiny forwarding contract that performs an internal ordinary `CALL` into the precompile, which remains valid because the precompile boundary is still direct.

## Verification
- Crate tests cover registry construction, duplicate-address rejection, dispatch routing, direct-call boundary enforcement, gas behavior, and revert mapping.
