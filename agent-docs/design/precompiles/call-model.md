# Precompile Call Model

## Summary
Whirlpool-owned stateful precompiles are direct-call-only at the precompile boundary. Ordinary `CALL` and `STATICCALL` into the precompile address are allowed. Delegate-style execution is rejected before business logic runs.

## Framework rule
`RegisteredPrecompile::new_stateful(...)` wraps each handler with a shared guard in `crates/precompiles/evm/src/lib.rs:47-60`. If `input.is_direct_call()` is false, the framework returns `NonDirectCall` in `crates/precompiles/evm/src/lib.rs:17-21` and `crates/precompiles/evm/src/lib.rs:73-92`.

## Why this design exists
- Prevent context confusion for stateful precompiles. Delegate-style paths blur which account/code identity is being executed.
- Keep safety policy centralized so every new stateful precompile inherits the same boundary check.
- Fail before precompile-local business logic starts, which keeps the policy easy to reason about.

## What is still allowed
A forwarding contract can make an ordinary internal `CALL` to the precompile address and remain valid, because the final hop is still direct. That path is covered in app-level verification tests in `crates/app/evm/app/src/executor.rs`.

## Example state model
The fee-pool precompile (`0x0000000000000000000000000000000000000102`) is the stateful reference model. Its write path (`withdraw`) mutates both journaled account balances and claim-ledger storage via EVM internals; its read paths expose fee-pool and per-recipient claimable balances.

## Design consequence
The fee-pool implementation proves the framework can host stateful precompiles without inventing a second storage system. The precompile reads and writes the same EVM state machine that proposal, verification, and RPC already use.
