# Step 4 Migration Map Pack

## Refactor-contract-table pointer
- Path: `.sisyphus/plans/split-state-interface-impl/.scratch/refactor-contract-table.md`
- Use section: `## Migration Steps -> Task Mapping`

## MIGRATION step IDs and labels
1. Lock interface surface in `state`.
2. Scaffold `state-memory` crate and workspace member.
3. Move concrete DB types + revm impl blocks to `state-memory`.
4. Rewire `app-evm` concrete imports/deps to `state-memory`.
5. Rewire `whirlpool-node` runtime wrapper to `state-memory`.
6. Remove transitional concrete exports from `state`.

## TESTS mapping
- Step 1: TB-001, TN-001, TN-002
- Step 2: TB-002
- Step 3: TB-003, TN-003
- Step 4: TB-004, TN-004
- Step 5: TB-005, TN-005
- Step 6: TB-006, TN-006

## Reference pointer
- `/home/dev/sahara/Runtime/skills/whiteboard-design/sisyphus-plan-from-refactor-docs/REFACTOR-TASK-REFERENCE.md`

## Crate CHANGES paths
- `docs/refactor/split-state-interface-impl/state/CHANGES.md`
- `docs/refactor/split-state-interface-impl/state-memory/CHANGES.md`
- `docs/refactor/split-state-interface-impl/app-evm/CHANGES.md`
- `docs/refactor/split-state-interface-impl/whirlpool-node/CHANGES.md`
