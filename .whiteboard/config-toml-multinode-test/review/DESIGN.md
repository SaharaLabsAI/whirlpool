# DESIGN Review

## Verdict: PASS

## Summary
Add config.toml file support to whirlpool-node and create a 4-node in-process integration test verifying P2P connectivity through block height growth and peer connection logging.

## Changes

### whirlpool-node (primary)
1. **Config TOML support**: New `TomlConfig` struct with serde Deserialize, `--config <path>` CLI flag, merge logic (CLI > TOML > defaults)
2. **Multi-validator config**: `--validator <hex_pubkey>` repeatable CLI arg, `validators` field in TOML and NodeConfig, replaces hardcoded single-validator
3. **Node startup extraction**: `start_node(NodeConfig) -> NodeHandle` extracted from main() for testability
4. **Peer connection logging**: INFO-level tracing when node startup wires peers

### integration-tests (test harness)
5. **4-node consensus test**: In-process tokio tasks, deterministic seeds, OS-assigned ports, shared validator set, tempdir isolation
6. **Block height verification**: All 4 nodes must reach height > 0, within ±1 of each other
7. **Peer connectivity proof**: Tracing log capture verifies peer connection events

## Design Decisions
- D1: Config layering via `toml` + `serde` — standard approach
- D2: --config and --validator CLI extensions — backward-compatible
- D3: Validators from config with single-signer fallback
- D4: In-process multi-node via tokio tasks — no Docker needed
- D5: Triple verification: height growth + height sync + peer logging

## Risks
- Test timing sensitivity (mitigated: 1s block interval, 60s timeout)
- In-process complexity (mitigated: standard tokio task pattern)

## Requirements Coverage
| Req | Covered By |
|-----|-----------|
| REQ-1: Config TOML | AC-1, AC-2, QA-1/2/3 |
| REQ-2: Config Layering | AC-2, INV-1 |
| REQ-3: Multi-Validator | AC-4, QA-4 |
| REQ-4: 4-Node Test | AC-5, BC-1/2/3/4 |
| REQ-5: Block Height | AC-6, BC-5/6 |
| REQ-6: P2P Connectivity | AC-5, AC-7, BC-7 |
| NFR-1: Backward Compat | AC-3 |
| NFR-2: Determinism | BC-2, INV-3 |
| NFR-3: Timeout | BC-3 |
