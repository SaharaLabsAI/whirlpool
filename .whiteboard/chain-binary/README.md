# Chain Binary Design

> Status: Draft
> Date: 2026-02-24
> Scope: Binary that wires `crates/consensus-core` + `crates/consensus-commonware`

## Goal

Define a chain binary that continuously finalizes **empty blocks** at a fixed cadence of **one block every 5 seconds**.

## Why this binary exists

- Provides a minimal executable chain for validating consensus integration.
- Uses the existing consensus abstraction (`consensus-core`) instead of coupling app logic to a backend.
- Uses the current Commonware backend (`consensus-commonware`) as the engine implementation.
- Produces deterministic empty blocks so behavior is easy to test and reason about.

## Document map

- `docs/design/chain-binary/architecture.md` - binary composition, crate wiring, runtime lifecycle.
- `docs/design/chain-binary/empty-block-cadence.md` - exact 5-second block production contract and verification rules.

## Non-goals for v0

- Transaction execution or mempool integration.
- Dynamic validator-set updates.
- RPC APIs beyond basic health/height visibility.
- State transitions beyond block indexing.
