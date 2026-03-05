# persistent-state - Execution Plan

## Objective

Generate an implementation-ready execution plan for persistent state using MDBX-backed `state-reth`, with incremental compile gates and explicit AC/INV coverage.

## Inputs

- Design docs: `.design-scratch/e2e/persistent-state-rethdb-20260305-1347/docs/`
- Proven AC: `.design-scratch/e2e/persistent-state-rethdb-20260305-1347/main/proven-ac.md`
- Workspace root: `/home/dev/sahara/web3/agent/playground/whirlpool`

## Wave Order (Required)

### Wave 1
- [ ] [01-state-trait-fallible-and-state-memory](tasks/01-state-trait-fallible-and-state-memory.md)

### Wave 2
- [ ] [02-state-reth-scaffold](tasks/02-state-reth-scaffold.md)

### Wave 3
- [ ] [03-state-reth-core-db-tables-codec](tasks/03-state-reth-core-db-tables-codec.md)

### Wave 4
- [ ] [04-state-reth-trie-state-root](tasks/04-state-reth-trie-state-root.md)

### Wave 5
- [ ] [05-state-reth-statedb-impl](tasks/05-state-reth-statedb-impl.md)

### Wave 6
- [ ] [06-state-reth-revm-impls](tasks/06-state-reth-revm-impls.md)

### Wave 7
- [ ] [07-state-reth-tests](tasks/07-state-reth-tests.md)

### Wave 8
- [ ] [08-consumer-fallible-migration-app-evm-rpc-eth](tasks/08-consumer-fallible-migration-app-evm-rpc-eth.md)

### Wave 9
- [ ] [09-whirlpool-node-reth-wiring](tasks/09-whirlpool-node-reth-wiring.md)

### Wave 10
- [ ] [10-integration-and-workspace-verification](tasks/10-integration-and-workspace-verification.md)

## Dependency Chain

`01 -> 02 -> 03 -> 04 -> 05 -> 06 -> 07 -> 08 -> 09 -> 10`

## AC Coverage Matrix

- `AC-1`: 02, 03, 04, 05, 06
- `AC-2`: 07
- `AC-3`: 01
- `AC-4`: 01
- `AC-5`: 08
- `AC-6`: 08
- `AC-7`: 09
- `AC-8`: 07, 10
- `AC-9`: 04, 07, 09, 10
- `AC-10`: 05, 07
- `AC-11`: 10
- `AC-12`: 10

## Invariant Coverage Matrix

- `INV-1`: 01
- `INV-2`: 01
- `INV-3`: 07, 10
- `INV-4`: 04, 07, 10
- `INV-5`: 05, 07, 10
- `INV-6`: 02, 06, 07, 09
- `INV-7`: 08
- `INV-8`: 04, 07, 09, 10

## QA Coverage Matrix

- `QA-1`: 07, 10
- `QA-2`: 10
- `QA-3`: 07

## Execution Constraints

- Use `nix develop --command <cmd>` for all cargo verification commands.
- Do not run cargo commands while authoring this plan.
- Do not modify design docs or `e2e-state.md`.
