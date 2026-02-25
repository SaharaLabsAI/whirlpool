# 30 - Dev Environment and Workflow

## Dev environment

Use the Nix flake shell when needed. The repository includes `flake.nix` with Rust tooling (including `cargo-nextest`).

## When in doubt

1. Follow the target subproject's own `AGENTS.md` and `llmdocs/`.
2. When designing new business logic or architecture, use the `rust-whiteboard-design-docs` skill first.
3. Keep changes small and focused.
4. Match formatting/lint/test expectations before considering work complete.
5. After completing code changes, use the `ctx-update-doc` skill to generate/update llmdocs for the affected crates.

## Git workflow

- Prefer atomic commits per completed unit of work.
- Follow commit message style already used in `git log`.
- Never commit secrets (for example `.env` or credentials files).
