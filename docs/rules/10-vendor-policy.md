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
- `llmdocs/`

When work touches a vendor project, follow that subproject's local instructions as authoritative.
