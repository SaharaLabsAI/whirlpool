# Crate Contract: whirlpool-node

## Responsibilities
- Construct rpc-mem service adapter with tx source (submit) and personality storage (read).
- Keep rpc server topology unchanged (eth + mem servers).

## Inputs
- Existing tx source and personality storage handles.

## Outputs
- Configured rpc-mem service instance supplied to rpc server startup.
