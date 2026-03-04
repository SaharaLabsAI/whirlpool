# Skill Digest — split-interface-impl

## Instance
- **Grounded**: instance_id = `split-interface-impl-20260304-1025` (refactor-state.md)
- **Grounded**: workspace_root = `/home/dev/sahara/web3/agent/playground/whirlpool` (Cargo.toml)
- **Grounded**: focus_crates = app, consensus, p2p, state, consensus-simplex, p2p-commonware, app-evm (user request, expanded)
- **Grounded**: depth = structural (default)

## Crate Structure (Grounded — filesystem + explore)
- `app/src/`: adapter.rs, error.rs, lib.rs, traits.rs, types.rs — traits.rs has Application+TxSource traits mixed with NoopTxSource/InMemoryTxPool impls
- `consensus/src/`: app.rs, block.rs, engine.rs, error.rs, event.rs, lib.rs, mock/, tests.rs — 6 traits across 4 files, pure interface already
- `p2p/src/`: errors.rs, lib.rs, mock.rs, traits.rs, types.rs — traits.rs already clean split
- `state/src/`: db.rs, error.rs, lib.rs — no trait exists, need StateDb introduced
- `consensus-simplex/src/`: adapter.rs, config.rs, engine.rs, lib.rs, mailbox.rs, sink.rs, tests.rs, types.rs — CommonwareBlock supertrait in types.rs, impls in adapter/engine
- `p2p-commonware/src/`: error.rs, lib.rs, peer_id.rs, provider.rs, receiver.rs, sender.rs, tests.rs — all impls, no local traits
- `app-evm/src/`: config.rs, error.rs, executor.rs, lib.rs — StateProvider trait + EvmApplication impl in executor.rs

## Dependency Layering (Grounded — Cargo.toml)
- Foundation: consensus, p2p, state (no upward deps)
- Middle: app (depends on consensus)
- Adapters: consensus-simplex (consensus+p2p+p2p-commonware), p2p-commonware (p2p), app-evm (app+state+consensus)

## Impact (Grounded — explore context files)
- 16 symbols across 6 impact areas
- 6 traits consolidate (consensus), 2 concrete types move (app), 2 new traits introduced (state, p2p-commonware)
- 8 migration steps in 3 batches (low→medium→high risk)

## Current Phase
- design (explore complete, synthesize pending)

## Unknowns
- (resolved by explore) All trait locations mapped
- (resolved by explore) Cross-crate dependencies fully traced
