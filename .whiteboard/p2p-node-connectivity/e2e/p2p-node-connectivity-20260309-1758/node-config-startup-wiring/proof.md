# Proof of Design: Sub-Intent B (node-config-startup-wiring)

## S0: Pre-conditions
- **Blockers**: PASS. No design gaps remain for Sub-Intent B.
- **Task Readiness**: `TASK_GEN_READY=true`. All required artifacts (crate-contracts, flows, tests, handoff) are finalized.
- **Traceability**: 
    - REQ-4 (CLI/Config) is fully mapped to `NodeArgs`, `NodeConfig`, and `parse_bootstrap_peer` in `config.rs`.
    - REQ-5 (Startup Wiring) is fully mapped to the `main.rs` refactor flow.
- **Source Verification**: Current hardcoded literals in `main.rs` and constants in `config.rs` have been identified as the baseline for `NodeConfig::default()`.

## S1: Design Coherence
Sub-Intent B establishes a structured configuration and startup boundary for the `whirlpool-node` crate. By introducing `clap`-derived CLI parsing and a tiered `NodeConfig` model, it replaces brittle hardcoded defaults with operator-controlled inputs.

- **Independence**: Sub-Intent B is independent of Sub-Intent C (consensus relay), as it only modifies the node's entry point and configuration surface.
- **Ordering**: B depends on the P2P provider builder API from Sub-Intent A to consume the new configuration fields (listen address, bootstrappers, etc.).
- **Completeness**: Every field required by the `p2p-commonware` builder (from Sub-Intent A) is now exposed via the CLI, fulfilling REQ-4 and REQ-5.

## S2: Invariants

### Local Invariants (INV-B-*)
- **INV-B-1**: `NodeConfig::default()` produces a runtime configuration identical to the pre-refactor hardcoded state (e.g., `whirlpool-dev` namespace, seed `0`, `data/` storage).
- **INV-B-2**: `parse_bootstrap_peer` rejects any input missing the `@` separator, having empty segments, or containing invalid hex/SocketAddr before the async runtime starts.
- **INV-B-3**: Every field in `NodeConfig` has a corresponding consumer in the startup sequence (builder, DB, RPC, or consensus).
- **INV-B-4**: All storage paths (state, runtime, mempool) derive deterministically from the single `data_dir` root.
- **INV-B-5**: `whirlpool-node` changes do not require or introduce public API modifications to `p2p-commonware` (beyond consuming the builder from Sub-Intent A).
- **INV-B-6**: CLI parsing and configuration normalization complete synchronously before `commonware_runtime::tokio::Runner` is initialized.
- **INV-B-7**: Network and consensus namespaces remain distinct configuration fields to preserve existing isolation.

### Cross-Sub-Intent Invariants (XINV-*)
- **XINV-1**: Sub-Intent B consumes the `CommonwareNetworkProviderBuilder` API as established in Sub-Intent A without modification.
- **XINV-2**: The `NodeConfig` structure is designed to be extensible, allowing for the addition of relay-specific parameters (e.g., channel caps) in Sub-Intent C.

## S3: Acceptance Criteria

### Testable Criteria (AC-B-*)
- **AC-B-1**: Launching `whirlpool-node` with no arguments results in a node that binds to ephemeral ports, uses seed `0`, and stores data in `./data/`, matching legacy behavior.
- **AC-B-2**: The value provided to `--listen-addr` is passed directly to the network builder's `listen_addr()` setter.
- **AC-B-3**: A valid `PUBKEY@HOST:PORT` string provided to `--bootstrap-peer` results in a `Bootstrapper` entry with the correct `ed25519::PublicKey` and `SocketAddr`.
- **AC-B-4**: Any malformed `--bootstrap-peer` string causes the process to exit with a descriptive error message during argument parsing/conversion.
- **AC-B-5**: Setting `--data-dir my-node` results in DBs opening at `my-node/state` and `my-node/mempool`, and runtime storage at `my-node/runtime`.
- **AC-B-6**: Multiple instances of `--bootstrap-peer` and `--dial-peer` accumulate into a single prioritized list for the network provider.
- **AC-B-7**: The `--validator-seed` value is used to derive the node's private key via `ed25519::PrivateKey::from_seed()`.

### QA Scenarios
- **QA-B-1 (Default Round-trip)**: Verify `NodeArgs::parse_from([])` converts to a `NodeConfig` that matches the hardcoded constants.
- **QA-B-2 (Full Customization)**: Launch node with all flags set (custom namespaces, addresses, and ports) and verify via logs/diagnostics that all values were respected.
- **QA-B-3 (Multi-node Local Test)**: Start two nodes on the same machine using different `--data-dir`, `--listen-addr`, and `--rpc-addr` to ensure no resource conflicts.

### Coverage Matrix
| Requirement | AC Reference | INV Reference |
|-------------|--------------|---------------|
| REQ-4       | AC-B-2, AC-B-3, AC-B-4, AC-B-6 | INV-B-2, INV-B-7 |
| REQ-5       | AC-B-1, AC-B-5, AC-B-7 | INV-B-1, INV-B-3, INV-B-4 |

## S4: Dependency Contract
- **clap 4.5**: Added to `whirlpool-node` for CLI parsing. Features: `derive`.
- **commonware-cryptography**: Used for `ed25519::PublicKey` and `PrivateKey` derivation.
- **p2p-commonware**: Consumed for the `Bootstrapper` type and `CommonwareNetworkProviderBuilder`.
- **Backward Compatibility**: No changes to existing public traits or workspace-wide dependencies.

## S5: Risk Assessment
- **Build Time**: Adding `clap` with `derive` increases compilation time for the `whirlpool-node` crate (Low impact).
- **Hex Encoding**: The parser assumes standard hex encoding for public keys. If the operator provides an incorrect format (e.g., base64), it will fail-fast as intended.
- **Seed-only Identity**: Relying on a `u64` seed for identity is sufficient for dev/test but will require extension for production keystores (Accepted limitation for this pass).
- **Namespace Collision**: While namespaces are now configurable, providing identical namespaces for two different clusters on the same network could lead to crosstalk (Operator responsibility).

## S6: Verdict
**PASS**
The design for Sub-Intent B is complete, coherent, and preserves all critical invariants. It provides a robust foundation for multi-node connectivity without regressing existing local development workflows.
