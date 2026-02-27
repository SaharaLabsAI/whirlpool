# Design: EVM Integration

Integrate EVM execution capability into Whirlpool consensus by introducing `app` (abstract application trait) and `app-evm` (concrete EVM backend) crates that delegate block proposal and verification to an EVM executor backed by reth.

## Loading tiers

### Tier 1 — Always load (essential context)
| File | Purpose | ~Lines |
|------|---------|--------|
| INTENT.md | Design scope and success criteria | 66 |
| CRATES.md | Crate index with purposes | 27 |
| domains/overview.md | Domain index table | 14 |
| wiring/overview.md | Wiring scope and domain index | 20 |

### Tier 2 — Load when working on related area
| File | Purpose | ~Lines | Load when... |
|------|---------|--------|--------------|
| architecture/overview.md | Subsystem map and flow index | 96 | implementing cross-crate features |
| tests/overview.md | Test strategy summary | 30 | writing tests |
| app/README.md | App crate contract pack | 197 | working on app crate |
| app-evm/README.md | App-evm crate contract pack | 250 | working on app-evm crate |
| domains/application.md | Application domain deep dive | 109 | working on application domain |
| domains/consensus.md | Consensus domain deep dive | 72 | working on consensus domain |
| domains/evm-execution.md | EVM execution domain deep dive | 108 | working on evm-execution domain |
| wiring/application.md | Application wiring matrix | 58 | wiring application capabilities |
| wiring/evm-execution.md | EVM execution wiring matrix | 18 | wiring evm-execution capabilities |

### Tier 3 — Load on-demand only
| File | Purpose | ~Lines | Load when... |
|------|---------|--------|--------------|
| WORKSPACE.md | Workspace map and build info | 88 | build/dependency questions |
| architecture/block-proposal.md | Block proposal 7-stage flow | 89 | implementing/debugging block proposal |
| architecture/block-verification.md | Block verification 7-stage flow | 91 | implementing/debugging block verification |
| architecture/consensus-app-bridge.md | Consensus-app adapter bridge mapping | 71 | understanding adapter patterns |
| architecture/node-startup.md | Node startup current vs proposed wiring | 82 | implementing/debugging node startup |
| tests/app-unit.md | App crate unit test contracts | 99 | writing unit tests for app |
| tests/app-evm-unit.md | App-evm crate unit test contracts | 93 | writing unit tests for app-evm |
| tests/application-integration.md | Application integration test contracts | 75 | writing integration tests for application |
| tests/evm-execution-integration.md | EVM execution integration test contracts | 74 | writing integration tests for evm-execution |
| tests/cross-crate-flows.md | Cross-crate end-to-end flow test outlines | 79 | writing cross-crate e2e tests |
