# INDEX — EVM Transaction Execution

> Design document set for implementing real EVM transaction execution in `app-evm`.
> Sub-Intent 1 of "produce EVM block for whirlpool-node".

## Loading Guide

Start with **Tier 1** for full context. Load Tier 2 for implementation details. Tier 3 is reference only.

## Tier 1 — Essential Context

| File | Lines | Purpose |
|---|---|---|
| `SUMMARY.md` | ~60 | **Start here.** Single overview of the entire design. |
| `INTENT.md` | ~70 | Objective, scope, success criteria, assumptions |
| `STRATEGY.md` | ~80 | Architecture direction, key decisions, risks, ordering |
| `CRATES.md` | ~10 | One-row-per-crate summary |
| `DOMAINS.md` | ~90 | Domain model, boundary contracts, invariants |

## Tier 2 — Implementation Details

| File | Lines | Purpose |
|---|---|---|
| `FLOWS.md` | ~100 | End-to-end propose/verify/commit flows with error paths |
| `TESTS.md` | ~80 | Test contracts mapped to success criteria |
| `app-evm/README.md` | ~80 | Per-crate contract: API, changes, test seams |
| `state/README.md` | ~60 | Per-crate contract: API, changes, test seams |

## Tier 3 — Reference

| File | Lines | Purpose |
|---|---|---|
| `WORKSPACE.md` | ~40 | Workspace map, dependency graph, entrypoints |
| `BLOCKERS.md` | ~50 | Active/resolved blocker registry |
