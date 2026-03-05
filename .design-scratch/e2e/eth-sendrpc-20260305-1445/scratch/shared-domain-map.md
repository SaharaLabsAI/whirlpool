# Shared Domain Map

## Grounded facts

| Domain | Owner Crate(s) | Core Types/Contracts | Evidence |
|---|---|---|---|
| Consensus orchestration | `consensus`, `consensus-simplex`, `whirlpool-node` | `ConsensusEngine`, `CommonwareEngine`, `FinalizationSink` | `crates/consensus/src/traits.rs`, `crates/consensus-simplex/src/engine.rs`, `crates/consensus-simplex/src/sink.rs`, `crates/whirlpool-node/src/main.rs` |
| Application execution contract | `app` | `Application`, `TxSource`, `EvmBlock`, `ExecutionResult` | `crates/app/src/traits.rs`, `crates/app/src/types.rs` |
| EVM execution implementation | `app-evm` | `EvmApplication`, `WhirlpoolEvmConfig`, `StateProvider` | `crates/app-evm/src/executor.rs`, `crates/app-evm/src/config.rs`, `crates/app-evm/src/traits.rs` |
| State interface + storage | `state`, `state-memory` | `StateDb`, `InMemoryStateDb` | `crates/state/src/traits.rs`, `crates/state-memory/src/db.rs` |
| Node binary wiring | `whirlpool-node` | `main`, `TestStateDb` | `crates/whirlpool-node/src/main.rs` |

## Domain boundaries
- `app` defines contracts; `app-evm` implements behavior.
- `state` defines state contract; `state-memory` implements data backend.
- `whirlpool-node` composes runtime artifacts and owns process lifecycle.
- No existing RPC domain crate is present.

## [PROPOSED] deltas

| Proposed Domain Addition | Owner | Rationale | Boundary Rule |
|---|---|---|---|
| Ethereum JSON-RPC serving (`eth` namespace) | `whirlpool-node` | Required for integration tests but operational/node concern, not consensus contract concern | Keep server + handlers in node binary layer |
| RPC request/response adaptation | `whirlpool-node` | Method signatures and serialization concerns are transport-facing | Avoid leaking jsonrpsee/alloy RPC types into `app` trait layer |
| Pending tx + receipt tracking index | `whirlpool-node` | Needed for `eth_sendRawTransaction` / `eth_getTransactionReceipt` consistency in tests | Backed by node-local shared state; no consensus trait change |

## UNKNOWNs
- Canonical source for transaction receipt details is absent today in app/state layers.
- Block reference semantics for all optional `block_id` args are not yet encoded in current APIs.
