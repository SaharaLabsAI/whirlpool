# Mempool Split: Interface / Implementation

> Split `mempool` crate into `mempool` (interface: trait + error) and `mempool-mdbx` (concrete MDBX implementation).

## Quick Info

| Field | Value |
|---|---|
| Design Docs | `docs/refactor/mempool-split-interface/` |
| Tasks | 7 |
| Waves | 7 (fully sequential) |
| Estimated Complexity | 3S + 4M |
| Status | `pending` |

## Plan

See [INDEX.md](./mempool-split-interface/INDEX.md) for the full task list and execution order.

## Key Constraints

- Every task must leave `cargo build --workspace` and `cargo test --workspace` passing.
- No vendor modifications.
- No behavior changes — pure structural refactor.
- Follow `state` / `state-memory` crate split pattern.
- All `cargo` commands via `nix develop --command <cmd>`.
