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
- `TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage)`: combined submit/read adapter backed by `PersonalityStorage::get_latest`.
- `SubmitPersonalityRequest` / `SubmitPersonalityResponse`: submit contract.
- `GetPersonalityRequest` / `GetPersonalityResponse`: finalized read contract.
- `start_rpc_server(service, addr)`: starts the mem RPC server and registers `mem_submitPersonality` and `mem_getPersonality`.

## Read Path
- Request field `personality_id` must be `0x`-prefixed hex.
- RPC layer decodes `personality_id` bytes before calling the service.
- Service reads finalized state through `PersonalityStorage::get_latest`.
- `None` maps to JSON-RPC `null`.
- `StoredPersonality` maps deterministically to response fields:
  - `tx_hash`, `signer`, `personality_id`, `markdown_hash` -> `0x` hex strings.
  - `block_height`, `nonce` -> unchanged numeric values.
  - `markdown` -> UTF-8 string, otherwise `RpcMemError::InvalidStoredMarkdown`.

## Error Contract
- Invalid hex prefix or hex decode failures are rejected before the service is called.
- Storage-backed read failures map to `RpcMemError::PersonalityRead`.
- Submit-only adapters return `RpcMemError::ReadCapabilityUnavailable` for reads.

## Tests
- `tests/get_personality_contract.rs`: RPC happy path, null/not-found, malformed hex rejection, and storage-backed service lookups.
- `tests/submit_contract.rs`: direct service submit behavior.
- `tests/submit_regression.rs`: RPC submit regression coverage.

## Status
Active. `mem_getPersonality` now supports finalized reads when the node injects a storage-backed adapter.
