# `core`

**Purpose**: component *interfaces* expressed as Rust traits.

Owns: traits that define the boundaries between modules (e.g. `Storage`, `Executor`, `Consensus`, `Network`, `Mempool`, `Rpc`).

Why: components depend on trait bounds, not concrete implementations.

Inputs/outputs: trait methods use `types` as the shared data model.

Depends on: `types` (and minimal `async`/error plumbing if needed).

Not in scope: concrete implementations, networking transports, wire formats.
