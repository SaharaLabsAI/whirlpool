# Precompile Wiring

## Summary
The design uses one registry, one factory seam, and one config seam so every EVM entrypoint gets the same precompile set.

## Wiring path
- Registry assembly: `crates/evm/precompiles/src/lib.rs:94-124`.
- EVM factory seam: `crates/evm/precompiles/src/lib.rs:127-163`.
- App-level config seam: `crates/evm/app/src/config.rs:27`, `crates/evm/app/src/config.rs:103-108`, and `crates/evm/app/src/config.rs:195-199`.

## Why the seam lives here
- Avoid vendor edits. Whirlpool-owned precompiles are injected through `PrecompilesMap`, not by changing upstream code; see `agent-docs/crates/evm-precompiles.md`.
- Avoid executor special-casing. Proposal and verification already go through `WhirlpoolEvmConfig`, so attaching the registry there keeps execution code simpler; see `agent-docs/crates/app-evm.md`.
- Keep RPC aligned with consensus execution. `eth_call` and estimation share the same `WhirlpoolEvmConfig`; see `agent-docs/crates/rpc-eth.md`.

## Validation strategy
- Unit coverage proves the config installs the registry: `crates/evm/app/src/config.rs:257-275`.
- Unit coverage proves replay accepts a block whose transaction reaches a precompile through a forwarding contract: `crates/evm/app/src/executor.rs:761-800`.
- Full-node coverage proves state-changing tx, read-only `eth_call`, and revert surfacing: `testing/integration-tests/tests/precompile_test_token.rs:397-488` and `agent-docs/crates/integration-tests.md:132-141`.

## Design consequence
If Whirlpool adds more custom precompiles, the intended extension point is the registry in `crates/evm/precompiles/src/lib.rs`, not ad hoc per-call-site wiring.
