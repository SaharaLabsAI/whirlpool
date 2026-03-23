# Proven Acceptance Criteria

## Acceptance Criteria
| ID | Description | Verification method |
|---|---|---|
| AC-1 | A valid `mem_submitPersonality` request is accepted, yields a deterministic tx hash, and enters the shared opaque-byte ingress path. | RPC/unit tests and mixed-ingress integration test |
| AC-2 | Oversize markdown is rejected deterministically before it can produce a finalized personality write. | Admission and validation tests |
| AC-3 | A payload whose declared markdown hash does not match markdown bytes is rejected deterministically in proposal/verification. | Validation and mixed-block tests |
| AC-4 | A mixed block containing one valid EVM transaction and one valid mem transaction preserves EVM execution semantics while accepting mem structural validation. | Mixed-block integration test |
| AC-5 | Personality data remains absent before finalization and appears only after finalization handling runs. | Finalization visibility integration test |
| AC-6 | Later finalized mem writes for the same `personality_id` replace earlier visible values. | Replacement semantics test |
| AC-7 | Restart behavior leaves the prototype in-memory store empty and documents the volatility explicitly. | Restart/volatility test and docs verification |
| AC-8 | The workspace and node wiring expose `app-mem` and `rpc-mem` as first-class crates without widening `rpc-eth` semantics. | Workspace membership and node wiring validation |

## QA Scenarios
| ID | Scenario |
|---|---|
| QA-1 | Mixed ingress happy path |
| QA-2 | Oversize markdown rejection |
| QA-3 | Hash mismatch rejection |
| QA-4 | Mixed block preservation |
| QA-5 | Finalization-only visibility |
| QA-6 | Replacement semantics |
| QA-7 | Prototype volatility |
| QA-8 | Workspace and node wiring audit |

## Invariants
| ID | Statement |
|---|---|
| INV-1 | `rpc-eth` remains Ethereum-only; mem submission is introduced only through `rpc-mem`. |
| INV-2 | The shared mempool remains opaque-byte based through `TxSource` and does not split into family-specific queues in v1. |
| INV-3 | `app-mem` validation is deterministic across proposal and verification for supported mem payloads. |
| INV-4 | Personality data is not externally visible before finalization and becomes visible only through finalization-time store writes. |
| INV-5 | Last-finalized-write-wins semantics per `personality_id` define visible prototype personality state. |
| INV-6 | v1 mem validation remains structural-only; cryptographic/Jolt verification is deferred and must not be implied by RPC or plan wording. |
