# AGENTS.md

This file is the top-level, tool-agnostic guide for AI coding agents working in this repository.

## Repository shape

- `vendor/` git submodules kept for **fast code reference**
- a Nix dev shell (`flake.nix`)

### `vendor/` is reference-only

The projects under `vendor/` are **not part of this project**. Do not change `vendor/**` unless the task explicitly asks you to.

The vendor submodules are:

- `vendor/commonware/` (Commonware monorepo)
- `vendor/reth/` (Reth)
- `vendor/alto/` (Alto)

Each vendor subproject has its own:

- `AGENTS.md` (agent instructions)
- `llmdocs/` (agent-oriented architecture/workflow docs)

## Start here (non-negotiable): read llmdocs first (when looking into vendor)

Before searching/reading any source code inside a vendor subproject, check whether it has `llmdocs/` and start from its index:

- Commonware: `vendor/commonware/llmdocs/index.md`
- Reth: `vendor/reth/llmdocs/index.md`
- Alto: `vendor/alto/llmdocs/index.md`

Suggested reading order (per subproject):

1) `llmdocs/index.md`
2) relevant `llmdocs/overview/*`
3) relevant `llmdocs/architecture/*`
4) relevant `llmdocs/guides/*`
5) relevant `llmdocs/reference/*`

## Nix dev environment

This repo includes a `flake.nix` intended to provide a Rust dev shell with tools like `cargo-nextest`.

## When in doubt

1) Follow the subproject's own `AGENTS.md` and `llmdocs/`.
2) Keep changes small and focused.
3) Match the subproject's formatting/lint/test expectations before considering work complete.

## User interaction

When you ask the user questions, address the user as **Bob**.

## Git workflow

- Every time you finish a todo item, create an **atomic commit** for that completed unit of work.
- Use `git commit --no-gpg-sign ...` (ignore GPG signing).
- Follow existing commit message style in `git log`.
- Never commit secrets (e.g. `.env`, credentials).
