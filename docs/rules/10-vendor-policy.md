# 10 - Vendor Policy

## `vendor/` is reference-only

Projects under `vendor/` are not part of this project's direct code surface.

Do not modify `vendor/**` unless the task explicitly asks for it.

Current vendor submodules:

- `vendor/commonware/`
- `vendor/reth/`
- `vendor/alto/`

Each submodule may include its own:

- `AGENTS.md`
- `agent-docs/`

When work touches a vendor project, follow that subproject's local instructions as authoritative.

## Tooling exclusions for `vendor/**`

Vendor code is excluded from this repository's local quality gates unless the task explicitly targets vendor changes.

- Do not run `cargo fmt` on `vendor/**`.
- Treat `cargo check` warnings originating from `vendor/**` as non-blocking.
- Treat `cargo clippy` warnings originating from `vendor/**` as non-blocking.
