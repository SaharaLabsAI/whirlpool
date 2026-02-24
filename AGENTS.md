# AGENTS.md

This is the top-level entrypoint for AI coding agents in this repository.

## Progressive reading order

Follow this sequence and stop once you have enough context for the task:

1. `agents/00-scope.md`
2. `agents/05-routing.md` — decision tree to skip docs you don't need
3. `agents/10-vendor-policy.md`
4. `agents/20-vendor-llmdocs-workflow.md`
5. `agents/30-dev-env-and-workflow.md`

## Fast rules

- Do not change `vendor/**` unless explicitly requested.
- For vendor investigations, start from each project's `llmdocs/index.md` before reading source code.
- When designing new business logic or architecture, you must use the `rust-whiteboard-design-docs` skill first.
- Keep changes small, focused, and aligned with local formatting/lint/test expectations.
- Never commit secrets.

## User interaction

When asking the user questions, address the user as **Bob**.
