# Design: EVM Block Production for whirlpool-node

Enable `whirlpool-node` to produce real EVM blocks with transaction execution instead of empty-block MVP behavior. The scope covers tx ingestion, EVM execution, block assembly/verification, and finalization-linked state commitment across `whirlpool-node`, `app-evm`, `app`, and `state`.

## Loading tiers

### Tier 1 — Always load (essential context)
| File | Purpose | ~Lines |
|------|---------|--------|
| INTENT.md | Design scope and success criteria | 59 |
| CRATES.md | Crate index with purposes | 30 |
| domains/overview.md | Domain index table | 21 |
| wiring/overview.md | Wiring scope and domain index | 23 |

### Tier 2 — Load when working on related area
| File | Purpose | ~Lines | Load when... |
|------|---------|--------|--------------|
| app-evm/README.md | Crate contract pack | 84 | working on app-evm |
| app/README.md | Crate contract pack | 98 | working on app |
| architecture/overview.md | Subsystem map and flow index | 187 | implementing cross-crate features |
| domains/application-layer.md | Domain deep dive | 61 | working on application-layer |
| domains/block-production.md | Domain deep dive | 64 | working on block-production |
| domains/evm-execution.md | Domain deep dive | 41 | working on evm-execution |
| domains/state-management.md | Domain deep dive | 39 | working on state-management |
| state/README.md | Crate contract pack | 80 | working on state |
| tests/overview.md | Test strategy summary | 54 | writing tests |
| whirlpool-node/README.md | Crate contract pack | 85 | working on whirlpool-node |
| wiring/application-layer.md | Wiring matrix for domain | 20 | wiring application-layer capabilities |
| wiring/block-production.md | Wiring matrix for domain | 21 | wiring block-production capabilities |
| wiring/evm-execution.md | Wiring matrix for domain | 17 | wiring evm-execution capabilities |
| wiring/state-management.md | Wiring matrix for domain | 20 | wiring state-management capabilities |

### Tier 3 — Load on-demand only
| File | Purpose | ~Lines | Load when... |
|------|---------|--------|--------------|
| BLOCKERS.md | Blocker registry and status | 68 | triaging unresolved design gaps |
| WORKSPACE.md | Workspace map and build info | 38 | build/dependency questions |
| architecture/block-finalization.md | Single end-to-end flow | 60 | implementing/debugging block-finalization |
| architecture/block-proposal.md | Single end-to-end flow | 71 | implementing/debugging block-proposal |
| architecture/block-verification.md | Single end-to-end flow | 63 | implementing/debugging block-verification |
| architecture/node-startup.md | Single end-to-end flow | 68 | implementing/debugging node-startup |
| architecture/state-commitment.md | Single end-to-end flow | 58 | implementing/debugging state-commitment |
| tests/app-evm-unit.md | Unit test contracts | 43 | writing unit tests for app-evm |
| tests/app-unit.md | Unit test contracts | 39 | writing unit tests for app |
| tests/block-production-integration.md | Integration test contracts | 37 | writing integration tests for block-production |
| tests/cross-crate-flows.md | End-to-end test outlines | 44 | writing cross-crate tests |
| tests/evm-execution-integration.md | Integration test contracts | 39 | writing integration tests for evm-execution |
| tests/state-unit.md | Unit test contracts | 37 | writing unit tests for state |
| tests/whirlpool-node-unit.md | Unit test contracts | 33 | writing unit tests for whirlpool-node |
