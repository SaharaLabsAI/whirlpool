# node-config-startup-wiring - Execution Plan

## TL;DR

| Field | Value |
|-------|-------|
| **Summary** | Complete Sub-Intent B by adding CLI-backed node startup config in `whirlpool-node` and wiring it through startup without changing `p2p-commonware` |
| **Deliverables** | Entry plan file, this index, and 5 serial task files covering clap dependency, config contract, peer normalization, startup wiring, and final verification |
| **Effort** | 5 tasks, 5 waves, estimated M complexity overall |
| **Parallel** | none; handoff requires strict serial execution |
| **Critical Path** | `01` -> `02` -> `03` -> `04` -> `05` |

## Context

### Original Request

Generate a complete `.sisyphus` execution plan for Sub-Intent B (`node-config-startup-wiring`) of `p2p-node-connectivity`, scoped only to `REQ-4` and `REQ-5`.

### Grounding Summary

- `agent/TASK_GEN_READY.md` is `READY` and explicitly marks Sub-Intent B as ready for task generation.
- `agent/handoff.md` is the ordering authority: `Cargo.toml` -> `config.rs` model -> `main.rs` wiring -> unit tests -> integration tests.
- `agent/crate-contracts/whirlpool-node.md` fixes the public config contract, required defaults, CLI flags, startup order, and fail-fast parsing behavior.
- `agent/tests.md` defines the exact unit and construction-level integration tests for `TST-REQ4-001..005` and `TST-REQ5-001..002`.
- The source baseline is narrow: `crates/whirlpool-node/src/config.rs` is only constants today, and `crates/whirlpool-node/src/main.rs` still hardcodes namespace, addresses, storage paths, validator seed, and RPC bind behavior.

### Scope Guardrails

- In scope: `crates/whirlpool-node/Cargo.toml`, `crates/whirlpool-node/src/config.rs`, `crates/whirlpool-node/src/main.rs`, and `crates/whirlpool-node/tests/startup_config.rs` if needed for clearer startup coverage.
- Out of scope: all design docs, `e2e-state.md`, any `p2p-commonware` source edits, config-file support, peer deduplication, multi-validator CLI expansion, and Sub-Intent A/C work.
- Every task must preserve no-flag backwards compatibility while making explicit CLI/config values available for startup consumers.

## Work Objectives

### Core Objective

Deliver an executable implementation plan for `REQ-4` and `REQ-5` so `whirlpool-node` accepts explicit startup networking inputs and threads those normalized values into the existing Commonware builder and node startup sequence.

### Deliverables

- Entry file: `.sisyphus/plans/node-config-startup-wiring.md`
- Plan index: `.sisyphus/plans/node-config-startup-wiring/INDEX.md`
- Task 01: `.sisyphus/plans/node-config-startup-wiring/01-add-clap-derive-dependency.md`
- Task 02: `.sisyphus/plans/node-config-startup-wiring/02-scaffold-node-config-contract.md`
- Task 03: `.sisyphus/plans/node-config-startup-wiring/03-add-peer-normalization-and-config-conversion.md`
- Task 04: `.sisyphus/plans/node-config-startup-wiring/04-rewire-startup-through-node-config.md`
- Task 05: `.sisyphus/plans/node-config-startup-wiring/05-final-verification-and-cleanup.md`

### Definition of Done

```bash
nix develop --command cargo build
nix develop --command cargo test
```

### Must Have

- Strict serial execution with one task per wave.
- Explicit traceability to `REQ-4`, `REQ-5`, and relevant `AC-B-*` identifiers in every task.
- Pre-task and post-task gates for every task.
- Exact file lists per task.
- TDD sequencing for config and startup-wiring work.
- Final verification that `main.rs` no longer owns hidden startup literals for in-scope fields.

### Must NOT Have

- No source edits outside `crates/whirlpool-node`.
- No tasks that modify `crates/p2p-commonware`.
- No design-doc or `e2e-state.md` edits.
- No scope expansion beyond `REQ-4` and `REQ-5`.
- No parallel task waves.

## Execution Strategy

### Parallel Execution Waves

- Wave 1: Task 01 only
- Wave 2: Task 02 only
- Wave 3: Task 03 only
- Wave 4: Task 04 only
- Wave 5: Task 05 only

### Dependency Matrix

| Task | Depends On | Wave |
|------|------------|------|
| 01-add-clap-derive-dependency | none | 1 |
| 02-scaffold-node-config-contract | 01-add-clap-derive-dependency | 2 |
| 03-add-peer-normalization-and-config-conversion | 02-scaffold-node-config-contract | 3 |
| 04-rewire-startup-through-node-config | 03-add-peer-normalization-and-config-conversion | 4 |
| 05-final-verification-and-cleanup | 04-rewire-startup-through-node-config | 5 |

## Task List

- [x] Task 1: Add clap derive dependency [**S**] -> [01-add-clap-derive-dependency.md](01-add-clap-derive-dependency.md)
- [x] Task 2: Scaffold NodeConfig contract [**M**] -> [02-scaffold-node-config-contract.md](02-scaffold-node-config-contract.md)
- [x] Task 3: Add peer normalization and config conversion [**M**] -> [03-add-peer-normalization-and-config-conversion.md](03-add-peer-normalization-and-config-conversion.md)
- [x] Task 4: Rewire startup through NodeConfig [**M**] -> [04-rewire-startup-through-node-config.md](04-rewire-startup-through-node-config.md)
- [x] Task 5: Final verification and cleanup [**S**] -> [05-final-verification-and-cleanup.md](05-final-verification-and-cleanup.md)

## Traceability Map

| Requirement | Acceptance Criteria | Planned Tasks |
|-------------|---------------------|---------------|
| `REQ-4` | `AC-B-2`, `AC-B-3`, `AC-B-4`, `AC-B-5`, `AC-B-6` | `02`, `03`, `04`, `05` |
| `REQ-5` | `AC-B-1`, `AC-B-2`, `AC-B-5`, `AC-B-7` | `03`, `04`, `05` |

## Final Verification

The finishing gate is Task 05, which requires a full `nix develop --command cargo build` and `nix develop --command cargo test` pass plus a scope audit confirming only `whirlpool-node` changed and `p2p-commonware` remained read-only.
