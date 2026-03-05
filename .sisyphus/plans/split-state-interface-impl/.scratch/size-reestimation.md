# Size Re-estimation

| Task | Target Files (count) | Cross-crate deps | Size Class | Result |
|---|---:|---:|---|---|
| 01-lock-interface-surface-in-state | 3 | 0 | S | PASS |
| 02-scaffold-state-memory-crate | 3 | 1 | M | PASS |
| 03-move-concrete-db-and-revm-impls | 4 | 2 | L | WARN (allowed) |
| 04-rewire-app-evm-to-state-memory | 6 | 2 | L | WARN (allowed) |
| 05-rewire-whirlpool-node-wrapper | 2 | 2 | S | PASS |
| 06-remove-transitional-concrete-paths | 4 (plus globbed consumer surfaces) | 2 | M/L boundary | WARN (allowed) |

Notes:
- No task is XL (8+ files) in explicit target lists.
- No L task has >=3 cross-crate dependencies.
