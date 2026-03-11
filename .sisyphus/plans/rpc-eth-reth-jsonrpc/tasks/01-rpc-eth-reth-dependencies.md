# Task 01: Add reth RPC/provider dependencies

## Status
- pending

## Dependencies
- none

## Wave
- Wave 1

## Complexity
- S

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Scope still matches `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/requirements.md`.
- [ ] No `vendor/**` edits are required for dependency resolution.
- [ ] `crates/rpc-eth/Cargo.toml` is still on the legacy JSON-RPC dependency set.
- [ ] This task remains commit-ready on its own.
- [ ] Artifact Registry entries for TST-1 through TST-12 remain pending.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/workspace.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/requirements.md`
- Codebase references:
  - `crates/rpc-eth/Cargo.toml`

## Vendor Usage Patterns
- Use local `path = "../../vendor/reth/..."` dependencies exactly as described in `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`.
- Do not edit `vendor/reth/**`; only point Cargo dependencies at those paths.

## What to do
1. Add the reth RPC/provider/pool/network/storage dependencies in `crates/rpc-eth/Cargo.toml` so later adapter modules can compile.
2. Remove or stop depending on obsolete direct legacy RPC dependencies that the design marks as replaced (`jsonrpsee`, `async-trait`, manual alloy RPC type wiring) unless they are still needed transitively for the public error surface.
3. Keep test scaffolding minimal: if `rpc-eth` has crate-level tests or doctests that need feature imports updated to keep the manifest compiling, apply only the minimum manifest-side changes.
4. Verify the manifest resolves for `rpc-eth` without introducing source edits outside the crate manifest.
5. Record the resolved dependency set in `.sisyphus/evidence/task-01-rpc-eth-reth-dependencies.md`.

## Mock Boundary
- No mocks required.
- Dependency resolution must stay at the manifest layer; adapter implementations are deferred to later tasks per `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`.

## AC trace
- REQ-1
- REQ-2
- REQ-3
- REQ-4

## Must NOT do
- Do not create `provider.rs`, `pool.rs`, `network.rs`, or `convert.rs` in this task.
- Do not modify `crates/whirlpool-node/**`.
- Do not touch `vendor/**`.
- Do not run workspace-wide cargo commands.

## Acceptance Criteria
- [ ] `crates/rpc-eth/Cargo.toml` contains the reth dependencies required by the design docs.
- [ ] Obsolete direct manifest entries are removed or justified.
- [ ] `nix develop --command cargo build -p rpc-eth` is documented as passing.
- [ ] `.sisyphus/evidence/task-01-rpc-eth-reth-dependencies.md` captures the commands and outcomes.
- [ ] The result is a coherent checkpoint suitable for a dedicated commit.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth --lib`
- [ ] Evidence file notes the dependency diff and gate results.
- [ ] Artifact Registry remains pending but unchanged.
- [ ] Create one dedicated git commit for this task before starting Task 02.

## Post-Task Reconciliation
- Confirm no `TST-*` status changes yet; this task only enables later behavior work.

## QA Scenarios
- Happy path: Cargo resolves all new local reth paths.
- Failure path: a removed legacy dependency is still required and must be restored with justification.
- Boundary case: manifest keeps `jsonrpsee` only if the public error type still needs a direct dependency.

## Evidence
- `.sisyphus/evidence/task-01-rpc-eth-reth-dependencies.md`
