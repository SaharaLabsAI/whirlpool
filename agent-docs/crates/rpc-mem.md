# rpc-mem: Personality JSON-RPC Server

## Summary
`rpc-mem` exposes the memory/personality JSON-RPC surface for Whirlpool. It keeps submit behavior in-process through `TxSource` and serves finalized personality reads through a storage-backed service adapter.

## Location
`crates/rpc/mem/`

## Dependency Boundaries
- `app`: `TxSource` trait for submit ingress.
- `app-mem`: `PersonalityMarkdownTx`, `SignatureScheme`, validation limits, and tx hashing.
- `state`: `StoredPersonality` and `PersonalityStorage` for finalized read semantics.
- `jsonrpsee`: HTTP JSON-RPC server and client/test types.

## Public API
- `MemoryTxService`: submit and read service boundary used by RPC handlers.
- `TxSourceMemoryTxService::new(tx_source)`: submit-only adapter; `mem_getPersonality` returns `ReadCapabilityUnavailable`.
- `TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage)`: combined submit/read adapter backed by `PersonalityStorage::get_latest` and `PersonalityStorage::get_by_tx_hash`.
- `start_rpc_server(service, addr)`: registers `mem_submitPersonality`, `mem_getPersonality`, and `mem_getTransactionByHash`.

## Tests
- `tests/get_personality_contract.rs`: RPC happy path, null/not-found, malformed input rejection, and storage-backed service lookups.
- `tests/submit_contract.rs`: direct service submit behavior.
- `tests/submit_regression.rs`: RPC submit regression coverage.

## Status
Active. Moved under `crates/app/execute/mem/`. `whirlpool-node` no longer wires this server directly.
