# Proof: P2P Provider Completeness

## S0 Pre-conditions

### Review-Lane Readiness
Confirm `review/DESIGN.md` verdict is PASS. The review confirms that the design set remains within the scope of REQ-1, REQ-2, and REQ-3, and that no blockers remain in `agent/blockers.md`.
Verified artifacts:
- `review/DESIGN.md`
- `review/INDEX.md`

### Task-Generation Readiness
Confirm `agent/TASK_GEN_READY.md` is READY. It specifies that blocker status is PASS and the scope is restricted to Sub-Intent A only. Crate contracts, flows, tests, and handoff are marked as complete.

### Completeness Check
All required design documents exist and contain valid content:
- `agent/strategy.md`: Defines approach for validator seeding, bootstrap injection, and channel preservation.
- `agent/crates.md`: Specifies file-level changes for `p2p-commonware` and `whirlpool-node`.
- `agent/workspace.md`: Details workspace integration and implementation ordering.
- `agent/domains.md`: Establishes ownership boundaries and domain models.
- `agent/blockers.md`: Confirms PASS status with no active blockers.
- `agent/crate-contracts/p2p-commonware.md`: Detailed contract for the primary P2P crate.
- `agent/crate-contracts/whirlpool-node.md`: Detailed contract for node integration.
- `agent/flows.md`: Documents validator seeding, bootstrap, and message routing flows.
- `agent/tests.md`: Maps requirements to `TST-*` test contracts.
- `agent/handoff.md`: Guides plan generation with dependency mapping.
- `agent/TASK_GEN_READY.md`: Formal readiness marker.
- `review/DESIGN.md`: Final review verdict.
- `review/INDEX.md`: Artifact manifest.

**S0 Verdict**: PASS

## S1 Design Coherence

### Requirement Coverage
- **REQ-1 (Validator Seeding)**: Covered by `strategy.md` (Key Decisions > Validator Seeding), `crates.md` (`provider.rs` changes), `crate-contracts/p2p-commonware.md`, `flows.md`, and `tests.md`.
- **REQ-2 (Bootstrap Peers)**: Covered by `strategy.md` (Key Decisions > Bootstrap Peer Injection), `crates.md` (`main.rs` and `provider.rs` changes), `crate-contracts/whirlpool-node.md`, `flows.md`, and `tests.md`.
- **REQ-3 (Channel Metadata)**: Covered by `strategy.md` (Key Decisions > Channel Metadata Preservation), `crates.md` (`receiver.rs` and `provider.rs` changes), `crate-contracts/p2p-commonware.md`, `flows.md`, and `tests.md`.

### Strategy-to-Implementation Alignment
Strategy decisions are accurately reflected in the implementation specifications:
- `strategy.md` mandates `CommonwareNetworkProviderBuilder::build` as the seeding point; `crates.md` specifies this change in `provider.rs`.
- `strategy.md` requires `CommonwareReceiver` to store and use a `Channel` field; `crates.md` details the struct and `recv()` updates in `receiver.rs`.
- `strategy.md` requires `main.rs` to supply validators and bootstrappers; `crates.md` confirms this update for `whirlpool-node/src/main.rs`.

### Domain Model Consistency
`domains.md` boundaries align with crate boundaries:
- `Peer identity domain` correctly spans `whirlpool-node` (derivation) and `p2p-commonware` (consumption).
- `Transport/channel domain` maintains `p2p` as the source of truths for channel constants while `p2p-commonware` handles translation.
- No leaks detected: `whirlpool-node` remains responsible for startup-time selection, while `p2p-commonware` encapsulates transport runtime assembly.

### Cross-Crate Consistency
The design ensures consistency across crates:
- `p2p-commonware` preserves the `p2p` stable contract by using its channel constants and message types.
- `whirlpool-node` uses the `p2p-commonware` builder according to the new specification without modifying the underlying transport traits.
- The use of `crate::traits::` inside `p2p-commonware` ensures consistent internal imports and adheres to existing project patterns.

**S1 Verdict**: PASS

## S2 Invariants

### Design Invariants
- INV-1: Trait contracts in `crates/p2p` must remain unchanged. | Source: `strategy.md` (Scope), `crate-contracts/p2p-commonware.md` | Verification: Inspect `crates/p2p/src/traits.rs` for modifications.
- INV-2: `CommonwareReceiver` must preserve the concrete `Channel` identifier assigned at construction through the `recv()` path. | Source: `strategy.md` (Key Decisions > Channel Metadata Preservation), `flows.md` (Flow 3) | Verification: Assert `NetworkMessage.channel` matches `self.channel` in `receiver.rs`.
- INV-3: Validator seeding must occur via `OracleHandle::update_validators` inside `CommonwareNetworkProviderBuilder::build` before provider handoff. | Source: `strategy.md` (Key Decisions > Validator Seeding), `flows.md` (Flow 1) | Verification: Verify `update_validators` call in `provider.rs` before returning from `build()`.
- INV-4: Bootstrap peers must be threaded into the Commonware `discovery::Config` during provider construction. | Source: `strategy.md` (Key Decisions > Bootstrap Peer Injection), `flows.md` (Flow 2) | Verification: Inspect `provider.rs` for `bootstrappers` injection into `discovery::Config::local`.
- INV-5: `crates/whirlpool-node/src/main.rs` remains the authority for startup-time selection of validators and bootstrappers. | Source: `domains.md` (Ownership Boundaries), `crate-contracts/whirlpool-node.md` | Verification: Confirm `main.rs` populates builder inputs.
- INV-6: Empty validator or bootstrapper lists must be handled safely without panics. | Source: `strategy.md` (Key Decisions), `domains.md` (Failure/Edge Cases) | Verification: Unit tests for empty input cases in `provider.rs`.
- INV-7: Channel constants (`VOTE`, `CERTIFICATE`, `RESOLVER`) from `crates/p2p` are the canonical identifiers for muxed lanes. | Source: `flows.md` (Flow 3), `crate-contracts/p2p-commonware.md` | Verification: Inspect `provider.rs` and `lib.rs` for use of `Channel::VOTE` etc.

**S2 Verdict**: PASS

## S3 Acceptance Criteria

### Derived Acceptance Criteria
- AC-1: `CommonwareNetworkProviderBuilder` applies `initial_validators` to the `OracleHandle` during `build()`. | REQ: REQ-1 | Verification: `TST-REQ1-001`
- AC-2: `CommonwareNetworkProviderBuilder` configures the discovery runtime with the provided `bootstrappers`. | REQ: REQ-2 | Verification: `TST-REQ2-001`
- AC-3: `CommonwareReceiver` emits `NetworkMessage` with the `channel` field set to its configured channel ID. | REQ: REQ-3 | Verification: `TST-REQ3-001`, `TST-REQ3-002`
- AC-4: `whirlpool-node` startup wiring populates the provider builder with active validator and bootstrap sets. | REQ: REQ-2 (integration) | Verification: `TST-REQ2-002`
- AC-5: `MultiplexReceiver` forwards messages without overwriting or "repairing" channel metadata. | REQ: REQ-3 | Verification: `TST-REQ3-003`

### QA Scenarios
- QA-1: Happy Path - Multi-node Discovery. | Covers: AC-2, AC-4 | Expected: Node started with valid bootstrappers discovers remote peers through Commonware discovery.
- QA-2: Happy Path - Consensus Admission. | Covers: AC-1 | Expected: Seeded validators are immediately recognized by the discovery oracle, allowing admission and message exchange.
- QA-3: Happy Path - Multiplexed Routing. | Covers: AC-3, AC-5 | Expected: Inbound messages on `VOTE`, `CERTIFICATE`, and `RESOLVER` lanes are correctly tagged and separable by downstream consumers.
- QA-4: Edge Case - Empty Seeds. | Covers: AC-1, AC-2 | Expected: Node starts successfully with empty validator/bootstrap lists, falling back to existing ephemeral/direct-only behavior without crashing.
- QA-5: Edge Case - Unknown Channel. | Covers: AC-3 | Expected: `CommonwareReceiver` correctly tags and emits messages for channels beyond the standard constants if configured.

### Test Contract Traceability
- `TST-REQ1-001` -> AC-1: Verifies validator seeding in builder.
- `TST-REQ1-002` -> AC-1: Verifies safe handling of empty validator set.
- `TST-REQ2-001` -> AC-2: Verifies bootstrap threading in builder.
- `TST-REQ2-002` -> AC-4: Verifies node-level wiring integration.
- `TST-REQ3-001` -> AC-3: Verifies `VOTE` channel tagging.
- `TST-REQ3-002` -> AC-3: Verifies `CERTIFICATE` and `RESOLVER` channel tagging.
- `TST-REQ3-003` -> AC-5: Verifies `MultiplexReceiver` transparency.

**S3 Verdict**: PASS

## S4 Dependency Contract

### Crate Dependency Analysis
1. **p2p-commonware**:
   - Current: Depends on `p2p`, `commonware-p2p`, `commonware-cryptography`, `commonware-runtime`, `commonware-utils`, and `commonware-stream`.
   - New: No new external or internal dependencies are required for Sub-Intent A.
   - Compatibility: Uses `commonware-p2p` paths via `../../vendor/commonware/p2p` which are pinned via git submodule.
   - Cycles: No new dependency edges; graph remains acyclic.
2. **whirlpool-node**:
   - Current: Depends on `p2p-commonware`, `app`, `consensus`, and `consensus-simplex`.
   - New: No new dependencies needed.
   - Compatibility: Already has a path dependency on `p2p-commonware`.
3. **p2p**:
   - Current: Stable abstraction crate with minimal dependencies (`bytes`, `serde`, `thiserror`, `tokio`).
   - Changes: No changes to `Cargo.toml`.

### Cross-Crate Interface Contract
- **p2p**: The stable contract in `src/traits.rs` (e.g., `NetworkProvider`, `NetworkMessage`) and `src/types.rs` (e.g., `Channel`) is preserved. `p2p-commonware` continues to implement these traits without requiring modifications to the upstream crate.
- **whirlpool-node**: The node consumes `p2p-commonware` through its builder. The proposed changes add `initial_validators` and `bootstrappers` setters to the builder, which is a backward-compatible extension for the node's `main.rs`.
- **consensus-simplex**: This crate consumes `NetworkMessage` from the P2P layer. By fixing the `channel` field in `p2p-commonware`, `consensus-simplex` will receive correct metadata without any changes to its own code or interface.

### Vendor Dependency Contract
- **OracleHandle::update_validators**: The vendor equivalent is `tracker::Oracle::update` (defined in `vendor/commonware/p2p/src/authenticated/discovery/actors/tracker/ingress.rs`). It accepts a `u64` index (epoch) and a `Set<C::PublicKey>`. This matches the "validator seeding" requirement.
- **discovery::Config**: The `commonware_p2p::authenticated::discovery::Config` struct contains a `bootstrappers` field and is used in both `recommended` and `local` constructor variants.
- **Sender/Receiver channel awareness**: Commonware's `Network::register` returns per-channel senders and receivers. `p2p-commonware` correctly wraps these, and the proposed fix ensures the assigned `Channel` ID is stored and emitted by the receiver.

**S4 Verdict**: PASS

## S5 Risk Assessment

### Proof Summary
- **S0 (Pre-conditions)**: PASS. All design artifacts and review lanes are ready.
- **S1 (Design Coherence)**: PASS. Requirements REQ-1, REQ-2, and REQ-3 are fully covered by the strategy and crate specs.
- **S2 (Invariants)**: PASS. Seven key invariants (INV-1 to INV-7) are defined and verifiable.
- **S3 (Acceptance Criteria)**: PASS. Five ACs and five QA scenarios cover happy paths and edge cases.
- **S4 (Dependency Contract)**: PASS. Dependency graph is stable, and vendor API usage is verified against source.

### Residual Risks
- **Bootstrap Connectivity**: While REQ-2 ensures bootstrap peers are injected into Commonware, actual connectivity depends on the remote bootstrap nodes being reachable and healthy.
- **Validator Set Synchronization**: REQ-1 seeds the oracle at startup, but subsequent validator set changes (Epoch transitions) are not handled in this sub-intent (deferred to later work).
- **Vendor API Evolution**: Future updates to the `commonware` vendor submodule could change the discovery or oracle interfaces, though this is mitigated by pinning.

### Mitigation Strategy
- **Bootstrap Connectivity**: Use `QA-1` (Multi-node Discovery) to verify that discovery actually occurs in a local network environment.
- **Validator Set Synchronization**: The design explicitly limits scope to "startup seeding". Later sub-intents will address epoch-based oracle updates once the baseline is stable.
- **Vendor API Evolution**: Strictly adhere to the vendor policy in `docs/rules/10-vendor-policy.md` and use `just lint` / `just test` after any vendor sync.

### Implementation Confidence
**Confidence**: HIGH
The design is grounded in actual vendor code (verified `tracker::Oracle` and `discovery::Config`) and requires minimal, high-leverage changes to `p2p-commonware` (storing a field, calling an existing method). The separation of concerns between `whirlpool-node` (authority) and `p2p-commonware` (executor) is cleanly maintained.

**S5 Verdict**: PASS

**Overall Proof Verdict**: PASS

