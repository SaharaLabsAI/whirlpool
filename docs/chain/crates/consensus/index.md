# `consensus`

**Purpose**: decide canonical/finalized blocks.

Owns: vote/finality rules, consensus state machine/driver, fork choice (if any).

Inputs: candidate blocks (headers + execution results) + peer votes/messages.

Outputs: finalized head + consensus events.

Depends on: `types`, `core` (traits), `storage`.

## Sub-pages

- [`types`](./types.md) — public config + event shapes
- [`driver`](./driver.md) — parent-level driver/backend abstractions
- [`simplex`](./backends/simplex/index.md) — Simplex backend (`vendor/commonware/consensus`)
