# INDEX — Real Simplex Consensus Wiring

## Loading Tiers

### Tier 1 — Essential Context (read first)

| File | Lines | Description |
|------|-------|-------------|
| `SUMMARY.md` | ~80 | Human review document — single overview of entire design |
| `INTENT.md` | ~60 | Objective, scope, success criteria, assumptions |
| `STRATEGY.md` | ~100 | Key decisions, risk areas, implementation ordering |
| `CRATES.md` | ~25 | In-scope and adjacent crate inventory |
| `DOMAINS.md` | ~80 | Domain model, wiring contracts, boundaries |

### Tier 2 — Implementation Detail (read when building)

| File | Lines | Description |
|------|-------|-------------|
| `FLOWS.md` | ~100 | Engine startup flow, block production cycle, shutdown |
| `TESTS.md` | ~60 | Unit/integration test contracts, acceptance criteria mapping |
| `consensus-simplex/README.md` | ~70 | Per-crate contract: engine wiring changes |
| `p2p-commonware/README.md` | ~50 | Per-crate contract: channel splitting |
| `whirlpool-node/README.md` | ~50 | Per-crate contract: binary wiring |

### Tier 3 — Reference (read when needed)

| File | Lines | Description |
|------|-------|-------------|
| `WORKSPACE.md` | ~50 | Workspace map, crate graph, build/read entrypoints |
| `BLOCKERS.md` | ~25 | Information gaps (all non-blocking) |

## Reading Guide

1. **Start with** `SUMMARY.md` for a complete overview
2. **Dive into** `STRATEGY.md` for key architectural decisions
3. **Implement from** `FLOWS.md` (engine startup flow) + per-crate READMEs
4. **Test against** `TESTS.md` acceptance criteria mapping
