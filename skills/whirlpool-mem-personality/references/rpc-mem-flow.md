# Whirlpool mem personality RPC flow

Use this reference when you need the exact request contract or to debug why a submitted personality is not visible yet.

## Endpoints

- Submit and read personality data on the node's dedicated mem RPC listener.
- Use the Ethereum RPC listener only for chain progress checks such as `eth_blockNumber`.
- Expect `mem_submitPersonality` and `mem_getPersonality` to be unavailable on the Ethereum RPC server.

## Request contract

`mem_submitPersonality` expects one object with these fields:

- `version`: current mem personality tx version. Use `SUPPORTED_PERSONALITY_TX_VERSION` from `app-mem` when constructing requests in repo code.
- `signer`: `0x`-prefixed hex bytes.
- `personality_id`: `0x`-prefixed hex bytes.
- `nonce`: unsigned integer.
- `markdown`: UTF-8 string.
- `signature_scheme`: `raw_secp256k1`.
- `signature`: `0x`-prefixed hex bytes.

`mem_getPersonality` expects one object:

- `personality_id`: `0x`-prefixed hex bytes.

## Read semantics

- `mem_submitPersonality` returns a deterministic `tx_hash` for the encoded mem tx.
- `mem_getPersonality` reads finalized state only.
- A missing or not-yet-finalized personality returns JSON `null`.
- After finalization, the result object contains:
  - `tx_hash`
  - `block_height`
  - `signer`
  - `personality_id`
  - `nonce`
  - `markdown`
  - `markdown_hash`

## Verification loop

1. Record the current block height from `eth_blockNumber` if you need a progress baseline.
2. Submit the personality payload through `mem_submitPersonality`.
3. Repeatedly call `mem_getPersonality` with the same `personality_id`.
4. Stop when the result becomes a JSON object instead of `null`.
5. Verify:
   - `tx_hash` matches the deterministic hash for the submitted payload.
   - `signer`, `personality_id`, `nonce`, and `markdown` match the request.
   - `markdown_hash` matches the submitted markdown bytes.
   - `block_height` is greater than the pre-submit baseline when that baseline was captured.

The integration test `testing/integration-tests/tests/rpc_mem_integration.rs` polls every 200ms for up to 30 seconds. Use the same pattern unless the task requires a different timeout.

## Common failure cases

- `mem_*` method called on the Ethereum RPC listener:
  - Expect a JSON-RPC method-not-found style error.
- `personality_id`, `signer`, or `signature` missing the `0x` prefix:
  - Treat as client input error.
- Long period of `null` from `mem_getPersonality`:
  - Check whether blocks are finalizing.
  - Check whether the node was started with a mem RPC address.
  - Check whether the submit went to the mem RPC listener instead of the Ethereum listener.

## Source of truth

- `agent-docs/crates/rpc-mem.md`
- `agent-docs/crates/whirlpool-node.md`
- `testing/integration-tests/tests/rpc_mem_integration.rs`
