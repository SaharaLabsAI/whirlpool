# Whirlpool Codex Personality Demo

This demo shows an end-to-end Whirlpool personality flow:

1. Persist a Codex personality document into `whirlpool-node` through `mem_submitPersonality` (direct mem RPC from the demo script).
2. Fetch the finalized markdown back out through `mem_getPersonality`.
3. Store fetched profiles in a shared local profile store.
4. Launch Codex with `--profile` resolved from that same store.
5. Optionally show Codex's built-in live style switching with `/personality`.

## Files

- `whirlpool-node-demo.toml`: static one-node config for the demo node.
- `codex_personality.sh`: thin shell wrapper that preserves the original command.
- `codex_personality.py`: end-to-end runner implementation.
- `profiles/`: ready-to-demo personality markdown variants.
- `.run/`: runtime state, logs, generated bootstrap files, and a workspace-local `CODEX_HOME`.
- `.run/fetched-profiles/`: canonical fetched-profile store used by `fetch`, `profiles`, and `launch-codex --profile`.

## Prerequisites

- `codex` on `PATH`
- `nix`
- `python3`
- `curl`

The script uses a workspace-local `CODEX_HOME` under `.run/codex-home` and symlinks:

- your existing `~/.codex/auth.json`
- your existing `~/.codex/config.toml`
- your existing `~/.codex/skills/.system`
- the repo skill at `skills/whirlpool-mem-personality`

That keeps the demo isolated from your primary Codex home while still making the Whirlpool skill available to the external `codex` process.

The `save` subcommand is a direct local RPC flow (no nested `codex exec`). It uses mem RPC only for personality operations: submit with `mem_submitPersonality`, then poll `mem_getPersonality` until finalized.

`start` now fails fast if the demo ports are already occupied instead of waiting for the RPC probe to time out.

## Demo Flow

From the repo root:

```bash
devtools/demo/personality/codex_personality.sh start
devtools/demo/personality/codex_personality.sh save --profile leon
devtools/demo/personality/codex_personality.sh save --profile generated-v1 --profile-file /tmp/generated-personality.md
devtools/demo/personality/codex_personality.sh fetch --profile leon-final
devtools/demo/personality/codex_personality.sh profiles
devtools/demo/personality/codex_personality.sh launch-codex --profile leon-final
```

`save --profile` accepts built-in profile values: `default`, `leon`, and `ada`.
`save --profile-file <path>` submits markdown from an arbitrary file (no code changes needed); optionally pair it with `--profile <name>` to control the registry alias.
Each profile is mapped to its own remote `personality_id` in `.run/fetched-profiles/registry.json`. Re-saving the same profile reuses that ID and increments nonce.

`fetch` selectors:

1. `--profile <name>`: resolve remote `personality_id` from local registry and fetch it.
2. `--personality-id <0x...>`: fetch directly by explicit remote ID.

When `--profile` is provided to `fetch`, that value is also used as the local fetched-profile alias.

`launch-codex --profile` resolves in this order:

1. Explicit filesystem path.
2. `.run/fetched-profiles/<value>.md`.
3. Matching index entry by `name`, `tx_hash`, `markdown_hash`, or `personality_id`.

If `launch-codex` is called without `--profile`, it uses the most recently fetched profile from the profile store, and falls back to built-in `default` when the store is empty.

`launch-codex --profile` also accepts built-in names (`default`, `leon`, `ada`) for direct launch without fetch.

After Codex opens, you can demonstrate built-in live switching with:

```text
/personality pragmatic
/personality friendly
/personality none
```

To inspect current state:

```bash
devtools/demo/personality/codex_personality.sh status
```

To stop the node:

```bash
devtools/demo/personality/codex_personality.sh stop
```

## Generated Artifacts

The script writes these runtime files under `devtools/demo/personality/.run/`:

- `whirlpool-node.log`
- `save-events.jsonl`
- `save-message.txt`
- `submit-response.json`
- `submit-tx.json`
- `submit-receipt.json`
- `fetch-response.json`
- `fetch-events.jsonl`
- `fetch-message.txt`
- `personality.md`
- `codex-bootstrap.md`
- `fetched-profiles/index.json`

`personality.md` contains the finalized markdown returned by `mem_getPersonality`.

`codex-bootstrap.md` wraps that finalized markdown as the initial prompt for a fresh Codex session.

## Demo Profiles

The `profiles/` directory includes two style variants designed for visibly different demo output:

- `leon.md`: steady, tactical, protective, concise.
- `ada.md`: sparse, precise, cool, high-signal.

These are written as "inspired by" character archetypes. Keep the demo grounded in response style, not roleplay.
