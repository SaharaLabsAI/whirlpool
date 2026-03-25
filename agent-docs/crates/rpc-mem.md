# rpc-mem: Personality JSON-RPC Server

## Summary
`rpc-mem` exposes the memory/personality JSON-RPC surface for Whirlpool. It keeps submit behavior in-process through `TxSource` and serves finalized personality reads through a storage-backed service adapter.

Location: `crates/rpc-mem/`

## Dependency Boundaries
- `app`: `TxSource` trait for submit ingress.
- `app-mem`: `PersonalityMarkdownTx`, `SignatureScheme`, validation limits, and tx hashing.
- `state`: `StoredPersonality` and `PersonalityStorage` for finalized read semantics.
- `jsonrpsee`: HTTP JSON-RPC server and client/test types.
- `serde`: request/response serialization.
- `hex`: deterministic `0x` hex encoding and decoding at the RPC boundary.
- `thiserror`: `RpcMemError`.

## Public API
- `MemoryTxService`: submit and read service boundary used by RPC handlers.
- `TxSourceMemoryTxService::new(tx_source)`: submit-only adapter; `mem_getPersonality` returns `ReadCapabilityUnavailable`.
- `TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage)`: combined submit/read adapter backed by `PersonalityStorage::get_latest` and `PersonalityStorage::get_by_tx_hash`.
- `SubmitPersonalityRequest` / `SubmitPersonalityResponse`: submit contract.
- `GetPersonalityRequest` / `GetPersonalityResponse`: finalized read contract.
- `GetTransactionByHashRequest` / `GetTransactionByHashResponse`: finalized tx-hash lookup contract.
- `start_rpc_server(service, addr)`: starts the mem RPC server and registers `mem_submitPersonality`, `mem_getPersonality`, and `mem_getTransactionByHash`.

## Read Path
- Request field `personality_id` must be `0x`-prefixed hex.
- Request field `tx_hash` must be `0x`-prefixed 32-byte hex.
- RPC layer decodes `personality_id` bytes before calling the service.
- RPC layer decodes `tx_hash` bytes before calling the service.
- Service reads finalized state through `PersonalityStorage::get_latest`.
- Service reads finalized tx entries through `PersonalityStorage::get_by_tx_hash`.
- `None` maps to JSON-RPC `null`.
- `StoredPersonality` maps deterministically to response fields:
  - `tx_hash`, `signer`, `personality_id`, `markdown_hash` -> `0x` hex strings.
  - `block_height`, `nonce` -> unchanged numeric values.
  - `markdown` -> UTF-8 string, otherwise `RpcMemError::InvalidStoredMarkdown`.
  - `version`, `signature_scheme`, and `signature` are also exposed on `mem_getTransactionByHash`.

## Error Contract
- Invalid hex prefix or hex decode failures are rejected before the service is called.
- Storage-backed read failures map to `RpcMemError::PersonalityRead`.
- Submit-only adapters return `RpcMemError::ReadCapabilityUnavailable` for reads.

## Tests
- `tests/get_personality_contract.rs`: RPC happy path, null/not-found, malformed hex rejection, and storage-backed service lookups.
- `tests/get_personality_contract.rs`: includes `mem_getTransactionByHash` happy path/null/malformed-hash coverage and storage-backed tx-hash lookups.
- `tests/submit_contract.rs`: direct service submit behavior.
- `tests/submit_regression.rs`: RPC submit regression coverage.

## Status
Active. `mem_getPersonality` and `mem_getTransactionByHash` support finalized reads when the node injects a storage-backed adapter.
