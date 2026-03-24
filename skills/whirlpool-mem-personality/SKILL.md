---
name: whirlpool-mem-personality
description: Use this skill when Codex needs to operate or verify Whirlpool's personality persistence flow through mem transactions on a running node. Apply it for tasks such as submitting a personality update with `mem_submitPersonality`, checking finalized visibility with `mem_getPersonality`, confirming that `mem_*` methods are exposed only on the mem RPC listener, or troubleshooting why a submitted personality has not appeared yet.
---

# Whirlpool Mem Personality

Use the existing Whirlpool mem-personality RPC flow. Treat `mem_submitPersonality` as mempool ingress and `mem_getPersonality` as a finalized-state read.

Read [references/rpc-mem-flow.md](references/rpc-mem-flow.md) when you need exact request fields, polling behavior, or troubleshooting details.

## Workflow

1. Confirm the target node exposes a dedicated mem RPC address.
2. Send `mem_submitPersonality` to the mem RPC listener, not the Ethereum RPC listener.
3. Treat a successful submit response as acceptance into the shared tx source, not proof of finalized persistence.
4. Poll `mem_getPersonality` with the same `personality_id` until it returns a non-null object.
5. Verify the finalized record matches the submitted `signer`, `personality_id`, `nonce`, `markdown`, and deterministic hashes.
6. If needed, query `eth_blockNumber` on the Ethereum RPC listener to confirm the chain is advancing while waiting.

## Operating Rules

- Treat personality visibility as finalized-only. `mem_getPersonality` may return `null` after a successful submit until consensus finalizes a block containing the mem tx.
- Expect `mem_*` methods only on the node's mem RPC listener. Calls to the Ethereum RPC server should fail.
- Keep request encoding strict: hex fields must be `0x`-prefixed, `markdown` must be UTF-8 text, and `signature_scheme` must be `raw_secp256k1`.
- Prefer verifying returned `tx_hash` and `markdown_hash` deterministically when the submitting payload is known.
- Use repo truth when available. The canonical behavior is covered by `agent-docs/crates/rpc-mem.md`, `agent-docs/crates/whirlpool-node.md`, and `testing/integration-tests/tests/rpc_mem_integration.rs`.

## Non-Goals

- Do not infer pending personality state from mempool admission alone.
- Do not claim restart-safe durability for personality reads beyond the current implementation.
- Do not route `mem_*` calls through the Ethereum RPC listener.
