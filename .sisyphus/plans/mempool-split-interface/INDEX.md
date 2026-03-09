# INDEX — Mempool Split Interface Plan

## Execution Order

| Wave | Task | Slug | Complexity | Target Crate(s) | Migration Step |
|---|---|---|---|---|---|
| 1 | 01 | scaffold-mempool-mdbx | S | mempool-mdbx (new) | Step 1 |
| 2 | 02 | add-mempool-store-trait | S | mempool | Step 2 |
| 3 | 03 | move-store-impl | M | mempool-mdbx, mempool | Step 3 |
| 4 | 04 | move-persistent-txpool | M | mempool-mdbx | Step 4 |
| 5 | 05 | move-integration-tests | S | mempool-mdbx, mempool | Step 5 |
| 6 | 06 | update-consumer | S | whirlpool-node | Step 6 |
| 7 | 07 | strip-mempool-interface | M | mempool | Step 7 |

## Dependencies

```
01 → 02 → 03 → 04 → 05 → 06 → 07
```

All tasks are strictly sequential. No parallelism.

## Design Doc References

- **INTENT**: `docs/refactor/mempool-split-interface/INTENT.md`
- **STRATEGY**: `docs/refactor/mempool-split-interface/STRATEGY.md`
- **IMPACT**: `docs/refactor/mempool-split-interface/IMPACT.md`
- **MIGRATION**: `docs/refactor/mempool-split-interface/MIGRATION.md`
- **TESTS**: `docs/refactor/mempool-split-interface/TESTS.md`

## Global Constraints

- All `cargo` commands: `nix develop --command cargo <subcmd>`
- No vendor modifications
- No behavior changes
- Each task must leave workspace building and tests passing
