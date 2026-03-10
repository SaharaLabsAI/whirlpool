# INDEX — EvmBlock TxSource

## File Inventory

### Tier 1 — Start here

| File | Lines | Description |
|---|---|---|
| `SUMMARY.md` | ~60 | Single-page overview of the entire design |
| `INTENT.md` | ~60 | Objective, scope, success criteria, assumptions |
| `STRATEGY.md` | ~70 | Architecture direction, key decisions, risks, implementation order |
| `CRATES.md` | ~10 | One row per in-scope crate |
| `DOMAINS.md` | ~70 | Domain model, entities, invariants, wiring |

### Tier 2 — Implementation detail

| File | Lines | Description |
|---|---|---|
| `FLOWS.md` | ~70 | End-to-end flows (submission, consumption, wiring) + impl slices |
| `TESTS.md` | ~70 | Unit and integration test contracts with success criteria mapping |
| `app/README.md` | ~50 | Per-crate contract for the `app` crate changes |

### Tier 3 — Reference

| File | Lines | Description |
|---|---|---|
| `WORKSPACE.md` | ~30 | Crate graph and key file entrypoints |
| `BLOCKERS.md` | ~15 | Blocker registry (empty — no blockers) |

## Reading Guide

1. **Quick understanding**: Read `SUMMARY.md` only (~2 min)
2. **Full design review**: Tier 1 files in order (~10 min)
3. **Implementation handoff**: Add Tier 2 files (~15 min total)
4. **Prior context**: See `docs/design/evm-tx-execution/` for Sub-Intent 1

## Prior Design

This design continues from `docs/design/evm-tx-execution/` (Sub-Intent 1: EVM execution engine).
TxSource implementation was explicitly deferred to this Sub-Intent 2.
