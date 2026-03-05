# app Design Contract

## 1) Role and purpose
- Role: `interface` crate in current workspace conventions.
- Purpose: define application-level contracts consumed by execution and node crates.
- Scope in this design: preserve existing interfaces while enabling node-local RPC integration through existing tx-pool and block/result exports.

## 2) Existing public API surface
- `pub trait Application` (`crates/app/src/traits.rs`)
  - `genesis`, `propose`, `verify` async contract used by adapter/executor paths.
- `pub trait TxSource` (`crates/app/src/traits.rs`)
  - `pending(&self) -> Vec<Vec<u8>>` drain contract for raw tx bytes.
- `pub struct InMemoryTxPool` (`crates/app/src/tx_source.rs`)
  - `new()`, `push(Vec<u8>)`, and `TxSource::pending()` implementation.
- `pub struct NoopTxSource` (`crates/app/src/tx_source.rs`)
  - empty source implementation.
- `pub struct EvmBlock` (`crates/app/src/types.rs`)
  - block envelope with roots, gas, timestamp, transactions.
- `pub struct ExecutionResult` (`crates/app/src/types.rs`)
  - execution outputs (`state_root`, `receipts_root`, `gas_used`, `receipt_count`).
- `pub enum ApplicationError` (`crates/app/src/error.rs`)
  - execution/verification/state error variants.

## 3) [PROPOSED] extensions
- None required for this minimum RPC scope.
- Rationale: node can satisfy required RPC methods by consuming existing `InMemoryTxPool` and node-held state handle without widening app trait contracts.
- Extension trigger (future): if multiple crates require shared RPC-facing tx/receipt abstractions, propose additive interface types here first.

## 4) Consumers
- `app-evm` consumes `Application`, `TxSource`, and app types (`crates/app-evm/src/executor.rs`).
- `whirlpool-node` consumes `ApplicationAdapter`, `InMemoryTxPool`, and app exports (`crates/whirlpool-node/src/main.rs`).
- Consensus adapter path indirectly depends on app contracts through `ApplicationAdapter`.

## 5) Migration notes
- No trait signature changes are proposed.
- No implementor migration required in this design pass.
- Implementation phase should avoid adding transport-specific types (`jsonrpsee`, HTTP, RPC request structs) to `app`; keep transport concerns in node layer.
