# INDEX

## Design Document Inventory

| File | Purpose | Lines | Tier |
|------|---------|-------|------|
| INTENT.md | User intent, success criteria, scope boundaries | ~70 | T1 (always load) |
| SUMMARY.md | Executive summary of design decisions and architecture | ~55 | T1 (always load) |
| STRATEGY.md | Implementation strategy with phased approach | ~120 | T1 (always load) |
| SHARED_CONTEXT.md | Grounded codebase facts for all phases | ~80 | T2 (load for planning) |
| EXPLORATION_DIGEST.md | Compressed exploration findings | ~35 | T2 (load for planning) |
| CRATES.md | Crate ownership and modification scope | ~30 | T2 (load for planning) |
| WORKSPACE.md | Workspace-level changes and dependency additions | ~40 | T2 (load for planning) |
| DOMAINS.md | Domain boundaries, wiring contracts, type layers | ~80 | T2 (load for planning) |
| FLOWS.md | Data flows, error paths, implementation slices | ~65 | T2 (load for planning) |
| TESTS.md | Test contracts (TC-001..TC-010) with coverage mapping | ~31 | T2 (load for planning) |
| BLOCKERS.md | Resolved and open blockers | ~10 | T3 (load on demand) |
| EXPLORATION.md | Full exploration notes | ~90 | T3 (load on demand) |
| app/README.md | App crate contract (no changes needed) | ~37 | T3 (load on demand) |
| whirlpool-node/README.md | Whirlpool-node crate contract (new modules) | ~50 | T2 (load for planning) |
| run-state.md | Sub-phase progress tracking | ~20 | Internal |

## Reading Guide

### For Planning (downstream plan generation)
Load T1 + T2 files. T1 gives you the "what" and "why". T2 gives you the "how" with enough detail to generate tasks.

### For Implementation (downstream code generation)
Load T1 + specific T2 files relevant to current slice. FLOWS.md maps slices to implementation order.

### For Review
Load T1 only. SUMMARY.md is self-contained for design review.
