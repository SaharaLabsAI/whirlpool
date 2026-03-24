# Proof

## S0: Pre-conditions

### Blocker and readiness check
- [x] All active `agent/blockers.md` items are resolved, accepted, or explicitly deferred.
- [x] `agent/TASK_GEN_READY.md` is handoff-ready for planning.
- Status: CLEAR
- Evidence: `.whiteboard/add-get-personality-to-rpc-mem/agent/blockers.md` (None), `.whiteboard/add-get-personality-to-rpc-mem/agent/TASK_GEN_READY.md` (Ready for prove phase: yes)

### Design completeness
- [x] `review/DESIGN.md` present and non-empty.
- [x] `review/INDEX.md` lists the expected review and agent docs.
- [x] No `[TODO]` or `[PLACEHOLDER]` markers in the reviewed design set.
- Status: COMPLETE
- Evidence: `.whiteboard/add-get-personality-to-rpc-mem/review/DESIGN.md`, `.whiteboard/add-get-personality-to-rpc-mem/review/INDEX.md`, grep scan across `.whiteboard/add-get-personality-to-rpc-mem/*.md`

### Evidence traceability
Every claim in this proof cites concrete design artifacts under `.whiteboard/add-get-personality-to-rpc-mem/` or Cargo manifests in the workspace. No unresolved `[UNGROUNDED]` claims remain.

## S1: Design Coherence

### Original approved intent
Add a read endpoint to `rpc-mem` so clients can fetch the latest finalized personality by `personality_id`, while preserving existing submit behavior (`review/alignment-digest.md`, `agent/requirements.md`).

### Sub-intents
Single-intent scenario. No decomposition is required because the scope is tightly coupled across one feature thread: RPC method surface + service contract + node wiring + tests (`agent/handoff.md`, `agent/workspace.md`).

### Independence argument
No split needed. The approved scope centers on one feature increment with one backward-compatibility guardrail (submit path unchanged), and all touched crates (`rpc-mem`, `whirlpool-node`, tests) are part of one coherent execution slice (`agent/crates.md`, `agent/handoff.md`).

### Ordering argument
Correct execution order is: define read contract in `rpc-mem` and schemas first, then wire read-capable adapter in `whirlpool-node`, then validate with rpc-mem tests including submit regression. This order follows crate boundaries and integration handoff constraints (`agent/handoff.md`, `agent/flows.md`, `agent/crate-contracts/whirlpool-node.md`).

### Completeness argument
REQ-1 through REQ-7 are fully represented by the design lane artifacts: method addition (REQ-1), service boundary contract (REQ-2), deterministic not-found behavior (REQ-3), submit preservation (REQ-4), state model reuse (REQ-5), deterministic validation (REQ-6), and test coverage plan (REQ-7) (`agent/requirements.md`, `agent/tests.md`, `review/DESIGN.md`).

## S2: Invariants

### Local invariants (per sub-intent)
| ID | Statement | Crate | Verification method | Evidence |
|---|---|---|---|---|
| INV-1 | `mem_submitPersonality` behavior and contract remain unchanged when `mem_getPersonality` is added. | rpc-mem | Regression test preserving existing submit fixture. | `agent/requirements.md` (REQ-4), `agent/tests.md` (TST-4), `agent/handoff.md` guardrails |
| INV-2 | `mem_getPersonality` reads finalized storage only via `PersonalityStorage::get_latest`, never pending/mempool data. | rpc-mem + state | Service integration tests asserting finalized-source semantics. | `agent/strategy.md`, `agent/requirements.md` assumptions, `agent/crate-contracts/state.md` |
| INV-3 | Request and response binary fields use deterministic hex validation/encoding at the rpc boundary. | rpc-mem | RPC unit tests for malformed hex and stable field encoding. | `agent/domains.md`, `agent/strategy.md`, `agent/tests.md` (TST-3) |
| INV-4 | Not-found behavior for unknown `personality_id` is deterministic and stable for clients. | rpc-mem | RPC tests for absent entry path. | `agent/requirements.md` (REQ-3), `agent/tests.md` (TST-2), `agent/crate-contracts/rpc-mem.md` |

### Cross-sub-intent invariants (XINV)
| ID | Statement | Origin sub-intent | Involved sub-intents | Verification method | Evidence |
|---|---|---|---|---|---|

None. This run has a single sub-intent (`main`), so no cross-sub-intent invariants are required.

### Invariant dependency graph
- INV-2 underpins INV-4 because deterministic not-found semantics depend on querying finalized storage only.
- INV-3 supports INV-2 and INV-4 by ensuring deterministic decode/encode behavior around lookup and response emission.
- INV-1 is orthogonal and protects backward compatibility of existing submit flow.

## S3: Acceptance Criteria

### Acceptance criteria
| ID | Description | Verification method | Sub-intent | Evidence |
|---|---|---|---|---|
| AC-1 | `rpc-mem` exposes `mem_getPersonality` JSON-RPC method accepting `personality_id` input. | RPC method registration test and request-shape validation test. | main | `agent/requirements.md` (REQ-1), `agent/flows.md` Flow 2, `agent/crate-contracts/rpc-mem.md` |
| AC-2 | `mem_getPersonality` uses a service-layer read contract with decoded ID bytes and no direct storage coupling inside handlers. | Service boundary tests and compile-time API wiring checks. | main | `agent/requirements.md` (REQ-2), `agent/domains.md` boundary contract, `agent/handoff.md` |
| AC-3 | Read path returns deterministic not-found behavior when personality is absent. | RPC not-found test. | main | `agent/requirements.md` (REQ-3), `agent/tests.md` (TST-2) |
| AC-4 | Existing `mem_submitPersonality` behavior remains unchanged after read-path addition. | Regression test for submit path fixture. | main | `agent/requirements.md` (REQ-4), `agent/tests.md` (TST-4), `agent/handoff.md` |
| AC-5 | Response payload maps from finalized `state::StoredPersonality` fields with stable encoding. | RPC happy-path response contract test. | main | `agent/requirements.md` (REQ-5), `agent/tests.md` (TST-1), `agent/crate-contracts/state.md` |
| AC-6 | Invalid or malformed `personality_id` input is rejected with deterministic rpc-mem validation mapping. | RPC malformed-input test. | main | `agent/requirements.md` (REQ-6), `agent/tests.md` (TST-3), `agent/crate-contracts/rpc-mem.md` |
| AC-7 | Test suite covers happy path and not-found behavior for new method. | Test inventory conformance check against planned TST IDs. | main | `agent/requirements.md` (REQ-7), `agent/tests.md` (TST-1, TST-2) |

### QA scenarios
| ID | Scenario | Steps | Expected result | AC covered |
|---|---|---|---|---|
| QA-1 | Found personality returns latest finalized entry | Seed storage with `StoredPersonality`; call `mem_getPersonality` with matching `personality_id`. | Response includes stable encoded fields (tx hash, block height, signer, personality_id, nonce, markdown, markdown_hash). | AC-1, AC-2, AC-5, AC-7 |
| QA-2 | Missing personality returns deterministic not-found | Ensure storage has no record for requested ID; call `mem_getPersonality`. | Stable not-found contract (null or explicit notFound shape once fixed by implementation). | AC-1, AC-2, AC-3, AC-7 |
| QA-3 | Malformed identity rejected | Call `mem_getPersonality` with missing `0x` prefix and invalid hex payload variants. | rpc-mem validation error mapping is deterministic. | AC-1, AC-6 |
| QA-4 | Submit regression remains green | Execute existing `mem_submitPersonality` fixture and compare response behavior with baseline. | Unchanged enqueue and tx-hash response path. | AC-4 |

### Coverage matrix
| AC ID | QA scenarios covering it | INV dependencies |
|---|---|---|
| AC-1 | QA-1, QA-2, QA-3 | INV-3 |
| AC-2 | QA-1, QA-2 | INV-2, INV-3 |
| AC-3 | QA-2 | INV-2, INV-4 |
| AC-4 | QA-4 | INV-1 |
| AC-5 | QA-1 | INV-2, INV-3 |
| AC-6 | QA-3 | INV-3 |
| AC-7 | QA-1, QA-2 | INV-2, INV-4 |

## S4: Dependency Contract

### Inter-crate dependencies introduced
| From crate | To crate | Dependency type | Justification |
|---|---|---|---|
| rpc-mem | state (via service adapter in node wiring) | Runtime service dependency (indirect) | Read path requires finalized personality lookup semantics exposed by `PersonalityStorage::get_latest`. |
| whirlpool-node | rpc-mem | Construction/wiring dependency | Node must instantiate rpc-mem service with both submit and read capabilities. |
| whirlpool-node | state / state-memory | Construction/wiring dependency | Adapter needs existing storage handle to back read RPC method. |

### External dependencies introduced
| Crate | Version | Purpose | Alternatives considered |
|---|---|---|---|

None required by design. Existing workspace dependencies appear sufficient (`crates/rpc-mem/Cargo.toml`, `crates/whirlpool-node/Cargo.toml`, `crates/state/Cargo.toml`).

### Feature flag changes
| Crate | Flag | Change | Impact |
|---|---|---|---|

None identified.

### Breaking changes
| Crate | Change | Migration path |
|---|---|---|

No breaking external API change is required for existing submit clients; new read method is additive.

## S5: Risk Assessment

### Identified risks
| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | Not-found response shape remains underspecified (`null` vs explicit object), causing client divergence. | Medium | Medium | Lock one explicit contract during implementation and codify in QA-2 + API docs/tests. |
| 2 | Trait/service boundary expansion accidentally regresses submit path behavior. | Medium | High | Keep INV-1 guardrail and enforce QA-4 regression before merge. |
| 3 | Node wiring adapter composes wrong storage source or stale handle. | Low | High | Bind adapter to `PersonalityStorage::get_latest` semantics and verify with QA-1/QA-2 integration tests. |
| 4 | Hex decode/encode inconsistencies surface across request/response fields. | Medium | Medium | Reuse existing deterministic validation pattern and cover malformed + happy path tests (QA-1, QA-3). |

### Biggest assumption
The design assumes `state::PersonalityStorage::get_latest` already provides the exact finalized-read semantics needed by rpc-mem without additional indexing/versioning. If this assumption is wrong, implementation may require state contract evolution and a design revisit.

### Unknowns
- Final not-found wire shape is still open in design artifacts and must be fixed in implementation-level contract tests.
- Exact error code/message mapping for storage backend failures in rpc-mem is not fully specified and should be pinned during coding.
