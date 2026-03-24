# Crate Inventory (Draft)

- `crates/rpc-mem`: JSON-RPC surface, method registration, request/response schemas, service traits.
- `crates/state`: canonical `StoredPersonality` and `PersonalityStorage` trait (`get_latest`).
- `crates/state-memory`: in-memory personality backend implementing `PersonalityStorage`.
- `crates/whirlpool-node`: node wiring; currently injects submit-only `TxSourceMemoryTxService` into rpc-mem.

Boundary note: `rpc-mem` currently depends on `TxSource` for writes only. Read endpoint likely needs a storage-backed service adapter in node wiring.
