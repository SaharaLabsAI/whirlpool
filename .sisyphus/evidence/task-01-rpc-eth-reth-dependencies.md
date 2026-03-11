# Task 01 Evidence: Add reth RPC/provider dependencies to rpc-eth

## Summary

Added reth RPC, provider, network, pool, evm, and consensus crate dependencies
to `crates/rpc-eth/Cargo.toml`. Kept all legacy dependencies since existing
source code still uses them (will be removed when source is rewritten in
Tasks 09-10).

## Changes

- **File**: `crates/rpc-eth/Cargo.toml`
  - Added 17 reth vendor crate path dependencies
  - Added `state-reth` internal crate dependency
  - Added `tokio`, `parking_lot` common deps for adapter code
  - Preserved all legacy deps (`jsonrpsee`, `alloy-*`, `async-trait`)

### Reth crates added

| Crate | Path | Purpose |
|-------|------|---------|
| reth-rpc | vendor/reth/crates/rpc/rpc | EthApi implementation |
| reth-rpc-builder | vendor/reth/crates/rpc/rpc-builder | RpcModuleBuilder |
| reth-rpc-eth-api | vendor/reth/crates/rpc/rpc-eth-api | EthApiServer traits |
| reth-rpc-eth-types | vendor/reth/crates/rpc/rpc-eth-types | RPC type helpers |
| reth-rpc-server-types | vendor/reth/crates/rpc/rpc-server-types | RpcModuleConfig |
| reth-provider | vendor/reth/crates/storage/provider | Provider bundle |
| reth-storage-api | vendor/reth/crates/storage/storage-api | Storage traits |
| reth-storage-errors | vendor/reth/crates/storage/errors | ProviderError |
| reth-chain-state | vendor/reth/crates/chain-state | CanonStateSubscriptions |
| reth-primitives-traits | vendor/reth/crates/primitives-traits | NodePrimitives |
| reth-ethereum-primitives | vendor/reth/crates/ethereum/primitives | EthPrimitives |
| reth-chainspec | vendor/reth/crates/chainspec | ChainSpec |
| reth-network-api | vendor/reth/crates/net/network-api | NetworkInfo |
| reth-transaction-pool | vendor/reth/crates/transaction-pool | TransactionPool |
| reth-evm-ethereum | vendor/reth/crates/ethereum/evm | EthEvmConfig |
| reth-consensus | vendor/reth/crates/consensus/consensus | Consensus traits |
| reth-tasks | vendor/reth/crates/tasks | TaskExecutor |

## Verification

- `cargo build -p rpc-eth`: **PASS** (compiled with CARGO_BUILD_JOBS=2)
- `cargo test -p rpc-eth --lib`: **PASS** (17/17 tests passed)
- 1 pre-existing warning: unused import `app::traits::TxSource`
- No vendor files modified
- No source files created or modified (manifest-only change)

## Timestamp

2026-03-11T06:00:00Z
