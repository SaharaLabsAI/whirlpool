# Workspace

## Current Shape
The workspace is organized around a clean layered path:
- `rpc-eth` accepts Ethereum transactions and forwards opaque bytes into `TxSource`.
- `app` defines shared traits and block types.
- `app-evm` proposes/verifies blocks and persists finalized block receipts through `BlockStorage`.
- `whirlpool-node` wires runtime, mempool, application, consensus, finalization sink, and RPC startup.
- `state` and `state-memory` provide storage-facing traits and in-memory components.

## Required Workspace Changes
- Add `crates/app-mem` and `crates/rpc-mem` to the workspace members in `Cargo.toml`.
- Keep the shared mempool and node process model: one node, one shared transaction ingress path, two RPC servers.
- Preserve the existing finalized block storage path while adding a sibling personality persistence path owned by node wiring.
- Keep transaction-family separation at crate boundaries: Ethereum behavior remains in `rpc-eth` and EVM execution remains in `app-evm`; mem/personality behavior lives in the new crates.

## Integration Points
- `crates/app/src/traits.rs`: shared raw-byte ingress remains the common boundary.
- `crates/app-evm/src/executor.rs`: mixed-transaction classification is the main application integration seam.
- `crates/whirlpool-node/src/node.rs`: add new crate wiring, store ownership, and dual RPC startup.
- `crates/whirlpool-node/src/persisting_sink.rs`: extend finalization persistence to include personality writes.
- `crates/state/src/lib.rs`: re-export any new personality storage trait if added under `state`.

## Invariants To Preserve
- EVM transaction behavior remains unchanged for existing Ethereum RPC clients.
- Non-finalized personality data is not visible in storage.
- Mempool remains generic and payload-agnostic.
- Consensus-visible validation for mem transactions stays deterministic across proposal and verification.
