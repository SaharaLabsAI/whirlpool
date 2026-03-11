# Workspace

## Change Topology
```
whirlpool-node (config.rs, main.rs, lib.rs)
    └── integration-tests (tests/multinode_consensus.rs)
```

## Integration Points
- whirlpool-node exposes `start_node(NodeConfig)` as library API → integration-tests consumes it
- NodeConfig carries validators list → passed to CommonwareConfig and network provider
- No changes to crate-to-crate interfaces between other crates

## Build Impact
- Workspace-level: `toml` and `serde` added to whirlpool-node — incremental, no breaking changes
- Integration tests gain `whirlpool-node` dep — pulls in full node dependency tree (already in workspace)
