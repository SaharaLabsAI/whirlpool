---
name: whirlpool-mem-personality
description: Use this skill when Codex needs to operate or verify Whirlpool's personality persistence flow through mem transactions on a running mem RPC service. Apply it for tasks such as submitting a personality update with `mem_submitPersonality`, checking finalized visibility with `mem_getPersonality`, or troubleshooting why a submitted personality has not appeared yet.
---

# Whirlpool Mem Personality

Use the existing Whirlpool mem-personality RPC flow when a mem RPC service is actually available. Treat `mem_submitPersonality` as mempool ingress and `mem_getPersonality` as a finalized-state read.

Read [references/rpc-mem-flow.md](references/rpc-mem-flow.md) when you need exact request fields, polling behavior, or troubleshooting details.

## Workflow

1. Confirm the target service exposes a mem RPC address. `whirlpool-node` no longer wires one by default.
2. Send `mem_submitPersonality` to the mem RPC listener, not the Ethereum RPC listener.
3. Treat a successful submit response as acceptance into the shared tx source, not proof of finalized persistence.
4. Poll `mem_getPersonality` with the requested `personality_id` until it returns a non-null object.
5. Verify the finalized record matches the submitted `signer`, `personality_id`, `nonce`, `markdown`, and deterministic hashes.
6. Query `mem_getTransactionByHash` on the mem RPC listener for the finalized `tx_hash`.
7. If needed, query `eth_blockNumber` on the Ethereum RPC listener to confirm the chain is advancing while waiting.

## Demo Profile Store Contract

For the demo tooling (`devtools/demo/personality/codex_personality.py`), use this shared local store:

- Profile directory: `devtools/demo/personality/.run/fetched-profiles/`
- Profile index: `devtools/demo/personality/.run/fetched-profiles/index.json`

When asked to fetch and persist a profile for later launch:

1. Resolve the target `personality_id` from the caller input (`profile` mapping or explicit `personality_id`).
2. Fetch finalized personality via `mem_getPersonality`.
3. Write raw markdown to the requested local profile path under the shared store.
4. Return metadata needed for index updates (`name`, `tx_hash`, `markdown_hash`, `personality_id`, `nonce`, `block_height`).

This contract keeps `fetch`, `profiles`, and `launch-codex --profile` aligned on the same profile artifacts.

## Create-New-Personality Policy

Interpret requests like "create new personality" as creating/submitting a new mem personality payload and local fetched-profile artifact, not adding a new built-in demo profile.

Allowed path:

1. Author markdown in a standalone file.
2. Print/render the full generated markdown to the user for review before any submit.
3. Ask for explicit confirmation to proceed with persistence.
4. Only after confirmation, submit with `save --profile-file <path>` (optionally with `--profile <alias>`).
5. Fetch/finalize and persist under `.run/fetched-profiles/`.

Prohibited path:

- Do not update `devtools/demo/personality/codex_personality.py` to add entries in `PROFILE_FILES`.
- Do not update `devtools/demo/personality/README.md` to document a new built-in profile.
- Do not treat personality creation as a built-in profile/code change workflow.
- Do not auto-submit newly generated personality content without explicit user confirmation.

## Operating Rules

- Treat personality visibility as finalized-only. `mem_getPersonality` may return `null` after a successful submit until consensus finalizes a block containing the mem tx.
- Expect `mem_*` methods only on a dedicated mem RPC listener. Calls to the Ethereum RPC server should fail.
- Keep request encoding strict: hex fields must be `0x`-prefixed, `markdown` must be UTF-8 text, and `signature_scheme` must be `raw_secp256k1`.
- Prefer verifying returned `tx_hash` and `markdown_hash` deterministically when the submitting payload is known.
- When reporting save results, include the submitted tx payload, finalized `tx_hash`, and `mem_getTransactionByHash` result.
- Use repo truth when available. The canonical behavior is covered by `agent-docs/crates/rpc-mem.md`, `agent-docs/crates/whirlpool-node.md`, and `testing/integration-tests/tests/rpc_mem_integration.rs`.

## Non-Goals

- Do not infer pending personality state from mempool admission alone.
- Do not claim restart-safe durability for personality reads beyond the current implementation.
- Do not route `mem_*` calls through the Ethereum RPC listener.
- Do not require Ethereum receipt checks for mem personality transaction verification.
- Do not create new built-in demo profiles via code/doc edits.
- Do not skip user preview/confirmation when creating new personality content.
