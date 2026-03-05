# Shared Crate Index

## Grounded facts

| Crate | Role | Key Public API / Types | Key Runtime Responsibility | Evidence |
|---|---|---|---|---|
| app | interface | `Application`, `TxSource`, `InMemoryTxPool`, `EvmBlock`, `ExecutionResult` | Trait boundary for execution + tx sourcing; tx pool implementation | `crates/app/src/traits.rs`, `crates/app/src/tx_source.rs`, `crates/app/src/lib.rs` |
| whirlpool-node | node/binary | `main` startup wiring; config constants in lib | Compose runtime/network/consensus/app; owns lifecycle | `crates/whirlpool-node/src/main.rs`, `crates/whirlpool-node/src/config.rs` |
| app-evm | implementation | `EvmApplication`, `WhirlpoolEvmConfig`, `SAHARA_CHAIN_ID`, `StateProvider` | Executes/validates transactions and mutates state DB | `crates/app-evm/src/executor.rs`, `crates/app-evm/src/config.rs`, `crates/app-evm/src/traits.rs` |
| state | interface | `StateDb` trait, `StateError` | Abstract state DB contract | `crates/state/src/traits.rs`, `crates/state/src/lib.rs` |
| state-memory | implementation | `InMemoryStateDb` | In-memory state backend implementing `StateDb` and revm db traits | `crates/state-memory/src/db.rs` |

## Interface crate audit
- `app` is clearly interface-led:
  - Exports `pub trait Application` and `pub trait TxSource` with high fan-out into `app-evm` and node wiring.
  - Also exports concrete helper types (`InMemoryTxPool`, `EvmBlock`, `ExecutionResult`) used cross-crate.
- `state` is also interface-led (`StateDb` trait), implemented by `state-memory`.
- For this design pass, required new RPC contracts can remain node-local; no mandatory interface extension in `app` is required to satisfy minimal method scope.

## [PROPOSED] deltas
- Keep RPC server contracts in `whirlpool-node` module(s) to preserve existing crate graph and avoid widening interface scope prematurely.
- Add dependency updates only to `crates/whirlpool-node/Cargo.toml` in implementation phase (`jsonrpsee`, likely `alloy-rpc-types-eth`/`alloy-serde` if needed by chosen method signatures).
- Optionally, future expansion could split a dedicated interface crate for RPC if method surface or consumers increase beyond node-local use.

## UNKNOWNs
- Exact receipt type to expose for `eth_getTransactionReceipt` in minimal implementation remains a design choice between full alloy receipt struct and reduced but compatible encoding.
- Exact block-tag semantics (`latest` vs `pending`) for `eth_getTransactionCount` in first iteration are to be fixed in strategy.
