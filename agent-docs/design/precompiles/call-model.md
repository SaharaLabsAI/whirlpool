# Precompile Call Model

## Summary
Whirlpool-owned stateful precompiles are direct-call-only at the precompile boundary. Ordinary `CALL` and `STATICCALL` into the precompile address are allowed. Delegate-style execution is rejected before business logic runs.

## Framework rule
`RegisteredPrecompile::new_stateful(...)` wraps each handler with a shared guard in `crates/app/execute/evm/precompiles/src/lib.rs:47-60`. If `input.is_direct_call()` is false, the framework returns `NonDirectCall` in `crates/app/execute/evm/precompiles/src/lib.rs:17-21` and `crates/app/execute/evm/precompiles/src/lib.rs:73-92`.

## Why this design exists
- Prevent context confusion for stateful precompiles. Delegate-style paths blur which account/code identity is being executed.
- Keep safety policy centralized so every new stateful precompile inherits the same boundary check.
- Fail before precompile-local business logic starts, which keeps the policy easy to reason about.

## What is still allowed
A forwarding contract can make an ordinary internal `CALL` to the precompile address and remain valid, because the final hop is still direct. That is the path used by the regression and full-node tests in `crates/app/execute/evm/app/src/executor.rs:619-651`, `crates/app/execute/evm/app/src/executor.rs:761-800`, and `testing/integration-tests/tests/precompile_test_token.rs:381-488`.

## Example state model
The validation/example `test-token` precompile lives at `0x0000000000000000000000000000000000000100`; see `crates/app/execute/evm/precompiles/src/test_token/mod.rs:10-13`. Its mint path rejects static execution and zero amounts, then mutates journaled account balance state through `internals_mut().balance_incr(...)`; see `crates/app/execute/evm/precompiles/src/test_token/impl.rs:23-53`. Its read path loads account balance from the EVM internals; see `crates/app/execute/evm/precompiles/src/test_token/impl.rs:56-80`.

## Design consequence
The example proves the framework can host stateful precompiles without inventing a second storage system. The precompile reads and writes the same EVM state machine that proposal, verification, and RPC already use.
