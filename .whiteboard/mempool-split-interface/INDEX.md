# INDEX — Mempool Interface/Implementation Split

## Reading Guide

### Tier 1 — Start Here (essential context)
| File | Lines | Description |
|---|---|---|
| [SUMMARY.md](./SUMMARY.md) | ~60 | Executive summary with narrative overview |
| [INTENT.md](./INTENT.md) | ~65 | Refactoring objective, scope, success criteria |
| [IMPACT.md](./IMPACT.md) | ~95 | Blast radius, call site analysis, dependency graph changes |
| [STRATEGY.md](./STRATEGY.md) | ~75 | Approach, crate design, error strategy, key decisions |

### Tier 2 — Implementation Details
| File | Lines | Description |
|---|---|---|
| [MIGRATION.md](./MIGRATION.md) | ~130 | 7 atomic migration steps with verification + rollback |
| [TESTS.md](./TESTS.md) | ~65 | 16 broken tests mapped to fixes, 2 new tests recommended |
| [mempool/CHANGES.md](./mempool/CHANGES.md) | ~40 | Per-crate: interface crate transformation |
| [mempool-mdbx/CHANGES.md](./mempool-mdbx/CHANGES.md) | ~40 | Per-crate: new implementation crate |

### Tier 3 — Blockers & Reference
| File | Lines | Description |
|---|---|---|
| [BLOCKERS.md](./BLOCKERS.md) | ~20 | Resolved and open blockers |

### Scratch (exploration artifacts, not for consumption)
| File | Description |
|---|---|
| `scratch/shared-impact-analysis.md` | Raw impact analysis data |
| `scratch/shared-dependency-graph.md` | Raw dependency graph data |
| `scratch/shared-test-coverage.md` | Raw test coverage data |
| `scratch/shared-module-structure.md` | Raw module structure data |
| `scratch/digests/` | Per-step digest summaries |
| `scratch/impact-context.md` | Synthesis input: impact |
| `scratch/migration-context.md` | Synthesis input: migration |
| `scratch/test-context.md` | Synthesis input: tests |
| `scratch/run-state.md` | Phase execution state |
| `scratch/STATE_DELTA.md` | Append-only state log |
| `scratch/MANIFEST.md` | Input/output tracking |
| `scratch/shared-refactor-splits.md` | Split assessment |
