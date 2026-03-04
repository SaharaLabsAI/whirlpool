# Refactor Splits

## Parent Intent
Split interface (trait definitions) from implementation for crates: app, consensus, p2p, and state.

## Threshold Analysis
- crates_in_scope: 4
- symbols_in_scope: 8
- depth_levels: 1
- cross_crate_boundaries: 4
- estimated_migration_steps: <= 15

## Split Decision
- required: no
- rationale: No threshold exceeded; scope is within structural intake bounds.

## Sub-Intent Tracking
- parent: split-interface-implementation
- children: none
- status: single-intent
