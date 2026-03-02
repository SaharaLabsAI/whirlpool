# INDEX — EVM Block Production Design Docs

> Generated for `docs/design/evm-block-production/`
> Total: 31 files, ~1632 lines

## Loading tiers

### Tier 1 — Always load (~170 lines)

Essential context for any work in this design scope.

| File | Lines | Description |
|---|---|---|
| `INTENT.md` | 59 | Design intent, scope, success criteria |
| `CRATES.md` | 30 | Crate index with purposes and dependencies |
| `domains/overview.md` | 21 | Domain boundary map (4 domains) |
| `wiring/overview.md` | 23 | Wiring scope and capability matrix |
| `BLOCKERS.md` | 33 | Active and resolved blockers registry |

### Tier 2 — Area-specific (~750 lines)

Load when working on related area.

| File | Lines | When to load |
|---|---|---|
| `architecture/overview.md` | 187 | Working on flows, implementation slices, or cross-crate integration |
| `tests/overview.md` | 54 | Working on test implementation or verifying coverage |
| `whirlpool-node/README.md` | 85 | Working on node binary or wiring |
| `app-evm/README.md` | 84 | Working on EVM execution or block proposal/verification |
| `app/README.md` | 98 | Working on Application trait or TxSource |
| `state/README.md` | 80 | Working on state management or InMemoryStateDb |
| `tests/cross-crate-flows.md` | 44 | Working on integration tests |
| `WORKSPACE.md` | 38 | Understanding build structure or dependency graph |

### Tier 3 — On-demand (~712 lines)

Load only when working on specific domain, flow, or test.

**Domain details:**

| File | Lines |
|---|---|
| `domains/block-production.md` | 64 |
| `domains/application-layer.md` | 60 |
| `domains/evm-execution.md` | 41 |
| `domains/state-management.md` | 39 |

**Wiring details:**

| File | Lines |
|---|---|
| `wiring/block-production.md` | 21 |
| `wiring/application-layer.md` | 19 |
| `wiring/evm-execution.md` | 17 |
| `wiring/state-management.md` | 20 |

**Architecture flows:**

| File | Lines |
|---|---|
| `architecture/block-proposal.md` | 71 |
| `architecture/block-verification.md` | 63 |
| `architecture/node-startup.md` | 68 |
| `architecture/block-finalization.md` | 60 |
| `architecture/state-commitment.md` | 58 |

**Test contracts:**

| File | Lines |
|---|---|
| `tests/app-evm-unit.md` | 43 |
| `tests/app-unit.md` | 39 |
| `tests/state-unit.md` | 37 |
| `tests/whirlpool-node-unit.md` | 33 |
| `tests/block-production-integration.md` | 37 |
| `tests/evm-execution-integration.md` | 39 |

## Reading guide

1. Start with **Tier 1** files to understand intent, scope, and blockers
2. Load `architecture/overview.md` for the full flow picture and implementation slices
3. Load per-crate READMEs relevant to your implementation target
4. Load specific architecture flow files when implementing that flow
5. Load test contracts when writing tests for a crate or flow

## Excluded from inventory

- `.design-scratch/` — ephemeral run artifacts, not design docs
