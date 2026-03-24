# Workspace Impact

## Crates Touched
- `crates/rpc-mem`: add read RPC method, request/response types, service contract expansion.
- `crates/whirlpool-node`: wire rpc-mem service with finalized personality storage dependency.
- `crates/state` / `crates/state-memory`: reused existing personality storage APIs; no required API expansion expected.

## Integration Notes
- Memory RPC remains split from Ethereum RPC.
- Personality read must use finalized store only.
