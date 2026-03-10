# Proven Acceptance Criteria — Sub-Intent B (node-config-startup-wiring)

## Acceptance Criteria

- **AC-B-1**: Node starts with no CLI arguments and behaves identically to pre-change binary (ephemeral ports, seed 0, data/ storage)
- **AC-B-2**: --listen-addr value reaches CommonwareNetworkProviderBuilder.listen_addr()
- **AC-B-3**: Valid PUBKEY@HOST:PORT via --bootstrap-peer parses to correct Bootstrapper(PublicKey, SocketAddr)
- **AC-B-4**: Malformed --bootstrap-peer fails with descriptive error before runtime starts
- **AC-B-5**: --data-dir propagates to all three storage sub-paths (state, runtime, mempool)
- **AC-B-6**: Multiple --bootstrap-peer and --dial-peer flags accumulate into single list
- **AC-B-7**: --validator-seed value used to derive ed25519 private key

## QA Scenarios

- **QA-B-1**: Default round-trip: NodeArgs::parse_from([]) → NodeConfig matches hardcoded defaults
- **QA-B-2**: Full customization: all flags set → all NodeConfig fields reflect provided values
- **QA-B-3**: Multi-node local test: two nodes with different data-dir/listen-addr/rpc-addr, no conflicts

## Invariants

- **INV-B-1**: NodeConfig::default() matches pre-refactor hardcoded state
- **INV-B-2**: parse_bootstrap_peer rejects malformed input before async runtime
- **INV-B-3**: Every NodeConfig field has corresponding startup consumer
- **INV-B-4**: Storage paths derive deterministically from data_dir
- **INV-B-5**: No p2p-commonware public API modifications
- **INV-B-6**: CLI parsing completes before runtime initialization
- **INV-B-7**: Network and consensus namespaces remain distinct

## Cross-Sub-Intent Invariants

- **XINV-1**: Sub-Intent B consumes builder API from Sub-Intent A without modification
- **XINV-2**: NodeConfig extensible for Sub-Intent C relay parameters

## Coverage Matrix

| Requirement | AC References | INV References |
|---|---|---|
| REQ-4 | AC-B-2, AC-B-3, AC-B-4, AC-B-6 | INV-B-2, INV-B-7 |
| REQ-5 | AC-B-1, AC-B-5, AC-B-7 | INV-B-1, INV-B-3, INV-B-4 |
