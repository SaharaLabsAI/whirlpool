# AGENTS.md

This is the top-level entrypoint for AI coding agents in this repository.

## Progressive reading order

Follow this sequence and stop once you have enough context for the task:

0. `llmdocs/index.md` — **read first** for architecture overview, crate map, and digested code information; often sufficient without reading source
1. `docs/rules/00-scope.md`
2. `docs/rules/05-routing.md` — decision tree to skip docs you don't need
3. `docs/rules/10-vendor-policy.md`
4. `docs/rules/20-vendor-llmdocs-workflow.md`
5. `docs/rules/30-dev-env-and-workflow.md`

## Fast rules

- **Before reading source code in any crate**, check if an `llmdocs/` directory exists for that crate/project and read its `index.md` first. llmdocs provide pre-digested architecture, API surfaces, and key patterns — use them to avoid expensive source-code crawling. Use the `ctx-read-doc` skill for structured llmdocs consumption.
- Do not change `vendor/**` unless explicitly requested.
- For vendor investigations, start from each vendor project's `llmdocs/index.md` before reading source code (see `docs/rules/20-vendor-llmdocs-workflow.md` for the full workflow).
- When designing new business logic or architecture, you must use the `rust-whiteboard-design-docs` skill first.
- Keep changes small, focused, and aligned with local formatting/lint/test expectations.
- Never commit secrets.
- All `cargo` commands must be run via `nix develop --command <cmd>` (cargo is not on PATH). Before marking any todo item or task complete, `cargo build` and `cargo test` must both pass. Fix any failures introduced by your changes before proceeding.
- Todo items must be implemented in their listed order. Do not skip ahead or work on items out of sequence.
- After completing code changes, always use the `ctx-update-doc` skill to generate/update llmdocs for the affected crates.

## User interaction

When asking the user questions, address the user as **Bob**.
