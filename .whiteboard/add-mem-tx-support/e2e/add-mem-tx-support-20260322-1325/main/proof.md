# Proof: Add mem tx support with dedicated app-mem and rpc-mem boundaries

## S0: Pre-conditions

### Blocker and readiness check
- [x] All active `agent/blockers.md` items are resolved, accepted, or explicitly deferred.
- [x] `agent/TASK_GEN_READY.md` is handoff-ready for planning.
- Status: CLEAR
- Evidence: `agent/blockers.md`, `agent/TASK_GEN_READY.md`, `review/INDEX.md`, `review/DESIGN.md`

### Design completeness
- [x] `review/DESIGN.md` present and non-empty.
- [x] `review/INDEX.md` lists the expected review and agent docs.
- [x] No `[TODO]` or `[PLACEHOLDER]` markers appear in the reviewed design set used for proof.
- Status: COMPLETE
- Evidence: `review/DESIGN.md`, `review/INDEX.md`, `agent/handoff.md`, `agent/TASK_GEN_READY.md`

### Evidence traceability
Every proof claim below is grounded in a specific design artifact section or a workspace file path cited inline.

## S1: Design Coherence

### Original approved intent
The approved scope is to add mem/personality transaction support with a dedicated `crates/app-mem` crate and a dedicated `crates/rpc-mem` crate, while keeping the shared mempool generic and making personality state visible only after finalization. Evidence: `review/alignment-digest.md`, `agent/requirements.md`, `review/DESIGN.md`.

### Sub-intents
| # | Title | Slug | Rationale |
|---|-------|------|-----------|
| 1 | Add mem tx support with dedicated app-mem and rpc-mem boundaries | main | The design explicitly records `Intent split decision: no split required`, and the work stays within one bounded module-scale cross-crate change set. Evidence: `agent/requirements.md`. |

### Independence argument
No split is needed because the approved design depends on a single integrated chain of changes: mem RPC ingress, mixed proposal/verification classification, and finalization-only storage are all linked through the same node composition boundary. Evidence: `agent/requirements.md`, `agent/flows.md`, `agent/handoff.md`, `agent/crate-contracts/whirlpool-node.md`.

### Ordering argument
The approved implementation order is: define `app-mem`, then add `rpc-mem`, then relax mixed proposal/verification behavior, then add prototype personality storage and finalization flushing, then wire dual RPC servers in `whirlpool-node`. This order matches the handoff build order and respects interface-before-implementation crate boundaries. Evidence: `agent/handoff.md`, `agent/crates.md`, `agent/crate-contracts/app-mem.md`, `agent/crate-contracts/rpc-mem.md`, `agent/crate-contracts/whirlpool-node.md`.

### Completeness argument
The design covers the full approved scope because it fixes new-crate ownership, preserves the generic `TxSource` ingress, defines deterministic mixed-family validation rules, and anchors finalization-only personality visibility in node-owned wiring. Evidence: `review/DESIGN.md`, `agent/strategy.md`, `agent/workspace.md`, `agent/domains.md`, `agent/flows.md`.

## S2: Invariants

### Local invariants (per sub-intent)
| ID | Statement | Crate | Verification method | Evidence |
|---|---|---|---|---|
| INV-1 | `rpc-eth` remains Ethereum-only; mem submission is introduced only through `rpc-mem`. | `crates/rpc-eth`, `crates/rpc-mem` | Review plan tasks and integration tests for RPC surface separation. | `review/DESIGN.md`, `agent/domains.md`, `agent/crate-contracts/rpc-mem.md`, `crates/whirlpool-node/src/node.rs` |
| INV-2 | The shared mempool remains opaque-byte based through `TxSource` and does not split into family-specific queues in v1. | `crates/app`, `crates/mempool-mdbx` | Unit/integration tests for shared ingress and mixed block handling. | `agent/requirements.md`, `agent/workspace.md`, `crates/app/src/traits.rs`, `crates/mempool-mdbx/src/persistent.rs` |
| INV-3 | `app-mem` validation is deterministic across proposal and verification for supported mem payloads. | `crates/app-mem`, `crates/app-evm` | Structural validation tests and mixed-block verification tests. | `agent/crate-contracts/app-mem.md`, `agent/flows.md`, `agent/domains.md` |
| INV-4 | Personality data is not externally visible before finalization and becomes visible only through finalization-time store writes. | `crates/whirlpool-node`, personality store crate | Finalization-only visibility tests and sink wiring checks. | `review/DESIGN.md`, `agent/flows.md`, `agent/crate-contracts/whirlpool-node.md`, `crates/whirlpool-node/src/persisting_sink.rs` |
| INV-5 | Last-finalized-write-wins semantics per `personality_id` define visible prototype personality state. | personality store crate | Replacement semantics tests across finalized blocks. | `review/DESIGN.md`, `agent/domains.md`, `agent/tests.md` |
| INV-6 | v1 mem validation remains structural-only; cryptographic/Jolt verification is deferred and must not be implied by RPC or plan wording. | `crates/app-mem`, `crates/rpc-mem` | Documentation and test review against deferred-scope guardrails. | `review/DESIGN.md`, `agent/strategy.md`, `agent/blockers.md`, `agent/crate-contracts/app-mem.md` |

### Cross-sub-intent invariants (XINV)
| ID | Statement | Origin sub-intent | Involved sub-intents | Verification method | Evidence |
|---|---|---|---|---|---|
| XINV-1 | Mixed proposal/verification must not regress existing EVM execution semantics while admitting valid mem transactions. | main | main | Mixed-block integration tests and EVM regression checks. | `agent/requirements.md`, `agent/flows.md`, `agent/tests.md`, `crates/app-evm/src/executor.rs` |
| XINV-2 | Node wiring owns cross-crate composition: dual RPC startup, shared `TxSource`, and finalization-time personality persistence are assembled centrally in `whirlpool-node`. | main | main | Node wiring task verification plus final integration test. | `agent/crate-contracts/whirlpool-node.md`, `agent/handoff.md`, `crates/whirlpool-node/src/node.rs` |

### Invariant dependency graph
- `INV-1`, `INV-2`, and `INV-3` support `XINV-1` because mixed-family correctness depends on separate RPC ownership, shared ingress, and deterministic classification.
- `INV-4` and `INV-5` support `XINV-2` because finalization-only visibility and replacement behavior both depend on node-owned sink composition.
- `INV-6` constrains both XINVs by limiting v1 behavior to structural validation only.

## S3: Acceptance Criteria

### Acceptance criteria
| ID | Description | Verification method | Sub-intent | Evidence |
|---|---|---|---|---|
| AC-1 | A valid `mem_submitPersonality` request is accepted, yields a deterministic tx hash, and enters the shared opaque-byte ingress path. | RPC/unit tests and mixed-ingress integration test. | main | `REQ-1`, `REQ-2`, `REQ-4`; `TST-001`; `agent/crate-contracts/rpc-mem.md`, `agent/flows.md` |
| AC-2 | Oversize markdown is rejected deterministically before it can produce a finalized personality write. | Admission and validation tests. | main | `REQ-8`; `TST-002`; `agent/tests.md`, `agent/crate-contracts/app-mem.md` |
| AC-3 | A payload whose declared markdown hash does not match markdown bytes is rejected deterministically in proposal/verification. | Validation and mixed-block tests. | main | `REQ-5`, `REQ-8`; `TST-003`; `agent/tests.md`, `agent/flows.md` |
| AC-4 | A mixed block containing one valid EVM transaction and one valid mem transaction preserves EVM execution semantics while accepting mem structural validation. | Mixed-block integration test. | main | `REQ-5`; `TST-004`; `review/DESIGN.md`, `agent/flows.md` |
| AC-5 | Personality data remains absent before finalization and appears only after finalization handling runs. | Finalization visibility integration test. | main | `REQ-6`, `REQ-7`; `TST-005`; `agent/flows.md`, `agent/crate-contracts/whirlpool-node.md` |
| AC-6 | Later finalized mem writes for the same `personality_id` replace earlier visible values. | Replacement semantics test. | main | `REQ-6`; `TST-006`; `agent/tests.md`, `agent/domains.md` |
| AC-7 | Restart behavior leaves the prototype in-memory store empty and documents the volatility explicitly. | Restart/volatility test and docs verification. | main | `REQ-6`, `REQ-9`; `TST-007`; `agent/tests.md`, `agent/blockers.md` |
| AC-8 | The workspace and node wiring expose `app-mem` and `rpc-mem` as first-class crates without widening `rpc-eth` semantics. | Workspace membership and node wiring validation. | main | `REQ-2`, `REQ-3`, `REQ-7`; `agent/workspace.md`, `agent/crates.md`, `Cargo.toml`, `crates/whirlpool-node/src/node.rs` |

### QA scenarios
| ID | Scenario | Steps | Expected result | AC covered |
|---|---|---|---|---|
| QA-1 | Mixed ingress happy path | Start node, submit valid personality tx, finalize containing block, inspect personality store after finalization. | Tx hash returned, mem tx included, finalized store updated only after finalization. | AC-1, AC-5 |
| QA-2 | Oversize markdown rejection | Submit payload over size limit through mem RPC or validation path. | Deterministic rejection, no finalized write. | AC-2 |
| QA-3 | Hash mismatch rejection | Submit payload with mismatched markdown hash. | Deterministic rejection during validation/proposal/verification. | AC-3 |
| QA-4 | Mixed block preservation | Build block with one valid EVM tx and one valid mem tx and run proposal/verification. | EVM semantics unchanged; mem tx validated structurally. | AC-4 |
| QA-5 | Finalization-only visibility | Inspect store before finalization, finalize block, inspect again. | No pre-finalization visibility; post-finalization visibility present. | AC-5 |
| QA-6 | Replacement semantics | Finalize two writes for same `personality_id` in later blocks. | Later finalized value is visible. | AC-6 |
| QA-7 | Prototype volatility | Restart node after finalized mem write and inspect store. | Store is empty after restart and docs call this out. | AC-7 |
| QA-8 | Workspace and node wiring audit | Verify workspace membership, dual RPC startup, and absence of mem methods in `rpc-eth`. | Crates integrated with intended ownership boundaries. | AC-8 |

### Coverage matrix
| AC ID | QA scenarios covering it | INV dependencies |
|---|---|---|
| AC-1 | QA-1 | INV-1, INV-2, INV-3 |
| AC-2 | QA-2 | INV-3, INV-6 |
| AC-3 | QA-3 | INV-3, INV-6 |
| AC-4 | QA-4 | INV-2, INV-3, XINV-1 |
| AC-5 | QA-1, QA-5 | INV-4, XINV-2 |
| AC-6 | QA-6 | INV-5, XINV-2 |
| AC-7 | QA-7 | INV-6 |
| AC-8 | QA-8 | INV-1, INV-4, XINV-2 |

## S4: Dependency Contract

### Inter-crate dependencies introduced
| From crate | To crate | Dependency type | Justification |
|---|---|---|---|
| `crates/rpc-mem` | `crates/app` | trait/service dependency | Submit canonical mem bytes through `TxSource` shared ingress. |
| `crates/rpc-mem` | `crates/app-mem` | API dependency | Reuse canonical payload validation and encoding rules. |
| `crates/app-evm` | `crates/app-mem` | API dependency | Classify/validate mem payloads during mixed proposal and verification. |
| `crates/whirlpool-node` | `crates/rpc-mem` | runtime wiring dependency | Start mem RPC alongside Ethereum RPC from the same node process. |
| `crates/whirlpool-node` | `crates/app-mem` | runtime wiring dependency | Flush derived finalized personality writes into the prototype store. |
| personality store crate | `crates/whirlpool-node` and/or `crates/state` | trait/backend dependency | Expose prototype in-memory storage for finalized personality visibility. |

### External dependencies introduced
| Crate | Version | Purpose | Alternatives considered |
|---|---|---|---|
| None required by the design proof | n/a | The design does not require a new external dependency to reach the approved prototype scope. | Reuse existing workspace crates and current runtime/storage patterns first. |

### Feature flag changes
| Crate | Flag | Change | Impact |
|---|---|---|---|
| None identified in the approved design set | n/a | No feature-flag changes are required by the proof. | Keeps workspace integration straightforward. |

### Breaking changes
| Crate | Change | Migration path |
|---|---|---|
| `crates/app-evm` | Mixed transaction handling replaces the assumption that all block txs are EVM-decodable. | Keep EVM execution behavior unchanged and add deterministic classification paths plus tests. |
| `Cargo.toml` workspace | Add `crates/app-mem` and `crates/rpc-mem` members. | Update workspace membership and compile/test the affected crate set. |

## S5: Risk Assessment

### Identified risks
| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | Mixed-family changes in `crates/app-evm/src/executor.rs` regress current EVM execution behavior. | Medium | High | Add mixed-block coverage early, preserve EVM-only happy paths, and gate progress on `TST-004`. |
| 2 | Prototype personality storage loses data on restart and may grow without durable limits. | Medium | Medium | Keep volatility explicit, test restart behavior, and defer durable storage out of v1 scope. |
| 3 | Structural-only signature handling is misunderstood as authenticity verification. | Medium | Medium | Keep RPC/docs language precise and ensure plan tasks preserve the deferred Jolt boundary. |
| 4 | Replay and replacement behavior remains under-specified for pending mem transactions. | Medium | Medium | Preserve future hooks in store and API design, but keep v1 acceptance focused on finalized-write semantics. |

### Biggest assumption
The design assumes the existing shared raw-byte ingress contract (`TxSource`) is stable enough to carry mixed EVM and mem families without splitting the mempool. If that assumption fails, the implementation would have to reopen design scope around queue partitioning and would no longer fit the approved v1 guardrails. Evidence: `agent/requirements.md`, `agent/workspace.md`, `crates/app/src/traits.rs`, `crates/mempool-mdbx/src/persistent.rs`.

### Unknowns
- The exact canonical mem payload codec is still an implementation choice inside the approved `app-mem` boundary.
- Replay protection by `(signer, nonce)` is left as a future hook and may need more storage keys later.
- The prototype personality store may live under `crates/state` or `crates/state-memory`; the design allows either so long as `BlockStorage` remains separate.
