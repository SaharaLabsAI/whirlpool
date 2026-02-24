# 30 - Dev Environment and Workflow

## Dev environment

Use the Nix flake shell when needed. The repository includes `flake.nix` with Rust tooling (including `cargo-nextest`).

## When in doubt

1. Follow the target subproject's own `AGENTS.md` and `llmdocs/`.
2. Keep changes small and focused.
3. Match formatting/lint/test expectations before considering work complete.

## Git workflow

- Prefer atomic commits per completed unit of work.
- Follow commit message style already used in `git log`.
- Never commit secrets (for example `.env` or credentials files).
