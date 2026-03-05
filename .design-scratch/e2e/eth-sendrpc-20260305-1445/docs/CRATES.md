# CRATES

| Crate | Role | Purpose | Existing API / Ownership | [PROPOSED] design scope | Evidence |
|---|---|---|---|---|---|
| app | interface | Application-facing execution and tx-source contract boundary | Owns `Application` + `TxSource` traits and exports `InMemoryTxPool`, `EvmBlock`, `ExecutionResult` | No trait extensions required for minimum RPC scope; RPC consumes existing `InMemoryTxPool` API | `crates/app/src/traits.rs::Application`, `crates/app/src/traits.rs::TxSource`, `crates/app/src/tx_source.rs::InMemoryTxPool`, `crates/app/src/lib.rs` |
| whirlpool-node | node/binary | Process lifecycle composition (runtime, network, consensus engine, app wiring) | Owns startup wiring and shared handles (`state_db`, `tx_pool`, `height`) | Add node-local `eth` JSON-RPC modules and lifecycle wiring after engine startup | `crates/whirlpool-node/src/main.rs::main`, `crates/whirlpool-node/src/config.rs` |

## Related crates (dependency context, not primary ownership targets)

| Crate | Role | Why relevant to this design | Evidence |
|---|---|---|---|
| app-evm | implementation | Defines chain id constant and execution behavior that RPC methods reflect | `crates/app-evm/src/config.rs::SAHARA_CHAIN_ID`, `crates/app-evm/src/executor.rs::EvmApplication::propose` |
| state | interface | Defines state DB access contract used by concrete state backend | `crates/state/src/traits.rs::StateDb` |
| state-memory | implementation | Supplies account balance/nonce reads and writes backing RPC read methods | `crates/state-memory/src/db.rs::InMemoryStateDb` |

## Role classification checks
- `app` qualifies as **interface**: high-fanout trait surface with minimal runtime orchestration.
- `whirlpool-node` qualifies as **node/binary**: owns runtime and integration composition.
- No new crate split is required for this intent because RPC is node-local and not a cross-crate public contract yet.
