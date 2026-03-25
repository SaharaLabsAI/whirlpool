# Whirlpool Codex Personality Demo

This demo shows two separate capabilities:

1. Persist a Codex personality document into `whirlpool-node` through the mem RPC flow, then fetch the finalized markdown back out and start a fresh Codex session from it.
2. Show Codex's built-in live style switching with `/personality`.

The custom Whirlpool-backed personality is applied on a fresh Codex launch. Arbitrary Whirlpool markdown is not treated as a supported hot-load mechanism for an already-running Codex session.

## Files

- `whirlpool-node-demo.toml`: static one-node config for the demo node.
- `demo_whirlpool_codex_personality.sh`: thin shell wrapper that preserves the original command.
- `demo_whirlpool_codex_personality.py`: end-to-end runner implementation.
- `.run/`: runtime state, logs, generated bootstrap files, and a workspace-local `CODEX_HOME`.

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

The `save` subcommand runs `codex exec` with `--sandbox danger-full-access` so the external Codex session can reach the local demo node at `127.0.0.1:9545` and `127.0.0.1:9645`. Without that, the Codex child process may sandbox its own local HTTP calls and fail before it reaches `mem_submitPersonality`.

`start` now fails fast if the demo ports are already occupied instead of waiting for the RPC probe to time out.

## Demo Flow

From the repo root:

```bash
devtools/demo/personality/demo_whirlpool_codex_personality.sh start
devtools/demo/personality/demo_whirlpool_codex_personality.sh save
devtools/demo/personality/demo_whirlpool_codex_personality.sh fetch
devtools/demo/personality/demo_whirlpool_codex_personality.sh launch-codex
```

After Codex opens, you can demonstrate built-in live switching with:

```text
/personality pragmatic
/personality friendly
/personality none
```

To inspect current state:

```bash
devtools/demo/personality/demo_whirlpool_codex_personality.sh status
```

To stop the node:

```bash
devtools/demo/personality/demo_whirlpool_codex_personality.sh stop
```

## Generated Artifacts

The script writes these runtime files under `devtools/demo/personality/.run/`:

- `whirlpool-node.log`
- `save-events.jsonl`
- `save-message.txt`
- `submit-response.json`
- `fetch-response.json`
- `personality.md`
- `codex-bootstrap.md`

`personality.md` contains the finalized markdown returned by `mem_getPersonality`.

`codex-bootstrap.md` wraps that finalized markdown as the initial prompt for a fresh Codex session.
