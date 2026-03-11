# SKILL_DIGEST

## Grounded

- **Workspace**: Whirlpool modular consensus framework (Rust). Crates: consensus, consensus-simplex, whirlpool-node, p2p, p2p-commonware, app, app-evm, state, state-memory, state-reth, mempool, mempool-mdbx, rpc-eth. (Source: Cargo.toml)
- **Current config**: NodeArgs (clap CLI) → NodeConfig conversion. No TOML support. Fields: listen_addr, dialable_addr, bootstrap_peer, dial_peer, validator_seed, rpc_addr, data_dir, max_message_size, network_namespace, consensus_namespace, block_interval_ms. (Source: config.rs)
- **Validator hardcode**: main.rs line 50: `validators = vec![signer.public_key()]` — single validator only. Must change to accept list from config. (Source: main.rs)
- **CommonwareConfig**: Takes validators: Vec<PublicKey>, plus timeouts, namespace, signer. Engine creates simplex Set from validators. (Source: consensus-simplex/src/config.rs)
- **P2P provider**: initial_validators(epoch, validators) and update_validators() — needs full validator set at startup. (Source: p2p-commonware)
- **Integration tests**: RPC-only, no multi-node/P2P/consensus tests. Uses in-memory state, not actual nodes. (Source: testing/integration-tests/)
- **Docker**: Empty directory, no multi-node infra. (Source: docker/)
- **Prior work**: Previous e2e (p2p-node-connectivity) completed: P2P provider completeness, node config & startup wiring, consensus relay activation — all done. (Source: .whiteboard/p2p-node-connectivity/)

## [PROPOSED]

- Add `toml` + `serde` (Deserialize) to whirlpool-node deps
- Create `TomlConfig` struct mirroring NodeConfig with serde attributes
- Add `--config <path>` to NodeArgs, load and merge with CLI precedence
- Add `validators` field (Vec<hex string>) to TOML schema
- 4-node in-process integration test using tokio tasks with tempdir isolation
- Block height polling via RPC eth_blockNumber or direct state query

## Unknowns

- None identified
