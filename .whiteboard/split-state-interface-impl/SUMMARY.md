# SUMMARY — Split State Interface/Implementation

This refactor package is finalize-ready for splitting `state` into an interface contract crate (`state`) and a concrete implementation crate (`state-memory`), with downstream rewiring in `app-evm` and `whirlpool-node`.

The design preserves runtime semantics and localizes risk to compile-time path/dependency churn. `StateDb`, `StateError`, and `DBErrorMarker` remain anchored in `state`, while `DbAccount`, `InMemoryStateDb`, and `revm` database impls move into `state-memory`.

`MIGRATION.md` provides six bounded, ordered steps with per-step verification and rollback. `TESTS.md` mirrors that sequence with explicit breakage mapping (`TB-001` through `TB-006`) and additive contracts (`TN-001` through `TN-006`), preserving the compilability invariant through the migration wave.

Safety-gate checks are clean:
- Circular dependency direction remains acyclic (`state-memory -> state` only).
- Compilability invariants are explicitly encoded in migration ordering and step gates.
- Public API movement is documented with downstream call-site impact and crate-level CHANGES.
- Test coverage maps each migration step to explicit contracts.
- No unresolved blockers are present.

## Key Decisions

- Keep interface contracts (`StateDb`, `StateError`, `DBErrorMarker`) in `state` to protect interface-only consumers and avoid contract-path churn.
- Move concrete storage/runtime integration (`DbAccount`, `InMemoryStateDb`, `DatabaseRef`, `Database`) into `state-memory` as a single ownership unit to reduce split-brain implementation risk.
- Rewire concrete consumers (`app-evm`, `whirlpool-node`) to `state-memory` while preserving trait/error contracts from `state`, minimizing behavioral risk.
- Enforce one-way dependency layering (`state-memory -> state`) and reject reverse-edge compatibility shortcuts.

These decisions support a **PASS** verdict because they satisfy all safety-gate criteria while preserving semantics and maintaining incremental compilability.

## Design Walkthrough

- **Intent and scope**: The split targets seven symbols across one architectural depth change, with threshold gate already passing without further decomposition.
- **Impact posture**: Primary breakage class is compile-time (imports/dependencies), with runtime behavior targeted as unchanged.
- **Execution strategy**: Interface lock first, implementation crate scaffold second, concrete move third, consumer rewiring fourth/fifth, cleanup sixth.
- **Verification spine**: Per-step `cargo check`/`cargo test` gates are defined and mirrored in test contracts.
- **Crate slices**: `state/CHANGES.md`, `state-memory/CHANGES.md`, `app-evm/CHANGES.md`, and `whirlpool-node/CHANGES.md` align with migration order.

## Final Verdict

**VERDICT: PASS — Design docs complete and internally consistent.**

- Document root: `docs/refactor/split-state-interface-impl`
- File count: 11 markdown files (including per-crate `CHANGES.md` and finalization artifacts)
- Migration step count: 6
- Test contract count: 6 (`TN-*`)
