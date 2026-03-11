# Strategy

## Approach
Extend the existing whirlpool-node config system with TOML file support and multi-validator configuration, then build a 4-node in-process integration test proving P2P connectivity through block height growth.

## Design Decisions

### D1: Config Layering via `toml` + `serde`
Add `toml` and `serde` (with Deserialize) dependencies. Create a `TomlConfig` struct with `Option<T>` fields mirroring `NodeArgs`. Load TOML first if `--config` is provided, then overlay CLI-provided values (non-default) on top. This gives standard CLI > TOML > Defaults precedence.

### D2: Extend NodeArgs with --config and --validator
Add `--config <path>` optional CLI arg. Add `--validator <hex_pubkey>` repeatable CLI arg. These feed into a new `validators` field on `NodeConfig`.

### D3: Validator Set from Config
Replace the hardcoded `validators = vec![signer.public_key()]` in main.rs with validators loaded from config. If no validators specified, fall back to `vec![signer.public_key()]` for backward compatibility.

### D4: In-Process Multi-Node Test Architecture
Use `whirlpool-node` as a library dependency in integration-tests. Create a `start_node(NodeConfig) -> JoinHandle` function that runs the main loop in a tokio task. Spin up 4 nodes with:
- Deterministic seeds (0, 1, 2, 3) for reproducible keys
- Port 0 (OS-assigned) for listen_addr, with actual port discovery for dialable_addr
- Tempdir for each node's data_dir
- All 4 validators in each node's config
- Bootstrap peers wired after port discovery

### D5: Block Height + Peer Connectivity Verification
Poll `eth_blockNumber` on each node's RPC endpoint. Assert height > 0 on ALL 4 nodes within timeout, and assert all heights within ±1 of each other (proves sync). Use 1s block intervals for fast test execution with 60s overall timeout. Add INFO-level tracing for peer connections in the node startup so tests can verify peer connection events. Log per-node height during the polling loop.

## Ordering
1. Config TOML support (config.rs changes) — standalone, testable
2. Multi-validator support (config.rs + main.rs) — builds on #1
3. Node startup refactor for testability (extract start_node fn from main) — enables #4
4. 4-node integration test — depends on #1-3

## Risk Mitigations
- Port collision: use port 0 everywhere
- Test flakiness: deterministic seeds + adequate timeouts + retry polling
- Backward compat: all new config fields are optional with existing defaults
