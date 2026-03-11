# Task 07 Evidence: Network Adapter

## Changes Made

### `crates/rpc-eth/src/network.rs` (NEW)
- Created `WhirlpoolNetwork` struct with `chain_id: u64`
- Implements `NetworkInfo` (local_addr, network_status, chain_id, is_syncing, is_initially_syncing)
- Implements `PeersInfo` (num_connected_peers → 0, local_node_record, local_enr)
- Implements `Peers` (all 12 methods return empty/noop — no real P2P layer)
- Static adapter: whirlpool is single-node, no peer management needed

### `crates/rpc-eth/tests/network_contract.rs` (NEW)
- TST-3a: `network_satisfies_rpc_builder_bounds` — type-level assertion for NetworkInfo + Peers + Clone
- TST-3b: `chain_id_round_trips` — constructor chain_id returned correctly
- TST-3c: `is_not_syncing` — both sync flags return false
- TST-3d: `network_status_returns_ok` — async status returns Ok
- TST-3e: `get_all_peers_returns_empty` — no peers in single-node

### `crates/rpc-eth/src/lib.rs`
- Added `pub mod network;`

### `crates/rpc-eth/Cargo.toml`
- Added `reth-network-peers` and `enr` dependencies

## Verification

- `nix develop --command cargo build -p rpc-eth` — ✅ passes
- `nix develop --command cargo test -p rpc-eth` — ✅ 29/29 tests pass (17 eth_handler + 5 network_contract + 3 pool_contract + 4 provider_contract)
