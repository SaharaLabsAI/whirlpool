# Proven Acceptance Criteria

## AC_VERSION
- ac_version: 2026-03-23T09:25:39Z

## Acceptance Criteria
| ID | Description | Verification method |
|---|---|---|
| AC-1 | `rpc-mem` exposes `mem_getPersonality` accepting `personality_id` input. | RPC method registration and request-shape tests. |
| AC-2 | `mem_getPersonality` uses a service-layer read contract over decoded ID bytes and keeps storage details out of handlers. | Service-boundary tests and compile-time wiring checks. |
| AC-3 | Absent personality lookups return a deterministic not-found contract. | RPC not-found path test. |
| AC-4 | Existing `mem_submitPersonality` behavior remains unchanged after read-path addition. | Submit regression test. |
| AC-5 | Found responses map from finalized `state::StoredPersonality` fields with stable encoding. | RPC happy-path response contract test. |
| AC-6 | Malformed `personality_id` input is rejected with deterministic rpc-mem validation mapping. | RPC malformed-input tests. |
| AC-7 | Planned tests cover happy and not-found paths for the new method. | Coverage check across TST-1/TST-2. |

## QA Scenarios
| ID | Scenario |
|---|---|
| QA-1 | Found personality returns latest finalized entry with stable field encoding. |
| QA-2 | Missing personality returns deterministic not-found contract. |
| QA-3 | Malformed identity input is rejected with validation mapping. |
| QA-4 | Existing submit path remains behaviorally unchanged. |

## Invariants
| ID | Statement |
|---|---|
| INV-1 | `mem_submitPersonality` behavior remains unchanged while adding `mem_getPersonality`. |
| INV-2 | Read path uses finalized storage semantics via `PersonalityStorage::get_latest` only. |
| INV-3 | Request/response binary fields keep deterministic hex validation/encoding. |
| INV-4 | Not-found behavior is deterministic for absent `personality_id`. |
