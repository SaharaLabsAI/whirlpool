# Workspace Integration

## Workspace Scope
- No workspace-member changes are required for synth design.
- `crates/rpc-eth` remains the integration crate for Ethereum JSON-RPC.
- `crates/whirlpool-node` remains the RPC startup caller and injection boundary.

## Planned `rpc-eth` Dependency Shape

### Additions (design target)
- reth RPC/build stack:
  - `reth-rpc`
  - `reth-rpc-builder`
  - `reth-rpc-eth-api`
  - `reth-rpc-eth-types`
  - `reth-rpc-server-types`
- reth provider/storage traits:
  - `reth-storage-api`
  - `reth-provider`
  - `reth-chain-state`
  - `reth-chainspec`
- reth network/pool traits:
  - `reth-network-api`
  - `reth-transaction-pool`
- reth primitives/config/consensus surfaces:
  - `reth-primitives-traits`
  - `reth-ethereum-primitives`
  - `reth-consensus`
- existing Whirlpool dependencies used by adapters:
  - `state`
  - `state-reth`
  - `app`
  - `app-evm`

### Removals or de-emphasis (post-migration expectation)
- Direct manual JSON-RPC server ownership in `rpc-eth` should move from current custom `jsonrpsee` trait/server implementation to reth-owned module composition.
- `jsonrpsee` and `async-trait` remain only if directly required after adapter migration; otherwise they should be transitive through reth crates.

## `whirlpool-node` Boundary Impact (Design)
- Replace current legacy `EthRpcContext` construction + `rpc::server::start_rpc_server(ctx, addr)` wiring with the new adapter-backed server entrypoint.
- Keep startup lifecycle ownership in `whirlpool-node`; only RPC server construction inputs change.
- Do not alter consensus, P2P, or application initialization ownership.

## Build and Path Assumptions
- All reth crates are vendored and consumed via local `path` dependencies.
- `vendor/**` remains read-only during this design phase.
- No workspace-level patching is planned for synth scope.

## Feature/Transport Scope
- HTTP transport is required for parity with current runtime startup path.
- WebSocket/IPC transport expansion is out of scope for this synth pass.
- RPC module selection focuses on standard `eth_*` path required by requirements (blob method explicitly unsupported).

## Compatibility Notes
- `state-reth::RethStateDb` already implements Whirlpool `StateDb` and `BlockStorage`, making it the primary adapter substrate.
- `app::traits::TxSource` remains the authoritative mempool ingress/pending source and is wrapped rather than replaced.
- `app-evm::WhirlpoolEvmConfig` and chain spec surfaces are expected inputs to reth RPC builder wiring.
