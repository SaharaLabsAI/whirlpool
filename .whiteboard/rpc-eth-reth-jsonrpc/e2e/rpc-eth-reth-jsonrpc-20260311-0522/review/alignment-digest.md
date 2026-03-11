# Alignment Digest

## Approved Intent
Wire reth's reth-rpc JSON-RPC server into rpc-eth by implementing adapter types (WhirlpoolProvider, WhirlpoolTxPool, WhirlpoolNetwork) that bridge our StateDb/BlockStorage/TxSource backends to reth's provider traits. Exclude blob tx support. Mirror reth's rpc-builder test patterns.

## Confirmed Scope
- **Primary target**: `crates/rpc-eth` (replace current stub implementation)
- **Integration touch**: `crates/whirlpool-node` (wire new server)
- **Read-only deps**: reth vendor RPC crates, `state-reth`, `app-evm`
- **No split required**: single focused crate replacement

## Approach Direction
1. Implement WhirlpoolProvider adapter (StateDb + BlockStorage → ~20 reth storage traits)
2. Implement WhirlpoolTxPool adapter (TxSource → TransactionPool)
3. Implement WhirlpoolNetwork adapter (minimal NetworkInfo)
4. Wire via reth's RpcModuleBuilder into EthApi
5. Stub blob methods with unsupported error
6. Integration tests using reth test patterns (NoopProvider-style)

## Risks
- **R1 (medium)**: Provider trait surface is large (~20 traits). Mitigation: start with NoopProvider pattern, implement real methods incrementally.
- **R2 (low)**: Blob code woven through reth — may need careful exclusion. Mitigation: return unsupported error at API level, don't modify reth internals.
- **R3 (low)**: Type conversions between our EvmBlock and reth block types. Mitigation: state-reth and app-evm already handle most conversions.

## Iteration Count
1 (pre-aligned via prior exploration)
