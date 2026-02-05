# `core`

**Purpose**: component *interfaces* expressed as Rust traits.

Owns: traits that define the boundaries between modules (e.g. `Storage`, `Executor`, `Consensus`, `Network`, `Mempool`, `Rpc`).

Depends on: `types` only.

## Sub-pages

- [`consensus`](./consensus.md) — consensus-facing application/verifier/reporter traits
