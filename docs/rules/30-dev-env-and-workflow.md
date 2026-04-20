# 30 - Dev Environment and Workflow

## Dev environment

Use the Nix flake shell when needed. The repository includes `flake.nix` with Rust tooling (including `cargo-nextest`).

`cargo` is **not** on `PATH` — it is only available inside the Nix dev shell. Always run cargo commands via `nix develop`:

```bash
nix develop --command cargo build
nix develop --command cargo test
nix develop --command cargo nextest run
```

Do **not** run bare `cargo` commands — they will fail because `cargo` is unavailable on PATH.

## When in doubt

1. Follow the target subproject's own `AGENTS.md` and `agent-docs/`.
2. When designing new business logic or architecture, use the `rust-whiteboard-design-docs` skill first.
3. Keep changes small and focused.
4. Match formatting/lint/test expectations before considering work complete.
5. After completing code changes, use the `ctx-update-doc` skill to generate/update agent-docs for the affected crates.

## Git workflow

- Prefer atomic commits per completed unit of work.
- Follow commit message style already used in `git log`.
- Never commit secrets (for example `.env` or credentials files).
