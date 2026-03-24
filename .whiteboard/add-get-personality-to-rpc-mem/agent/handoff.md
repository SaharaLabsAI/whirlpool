# Handoff

The design is ready for proof-phase argumentation and then plan generation.

## Core Decision
Implement `mem_getPersonality` in `rpc-mem` with finalized storage semantics.

## Required Implementation Areas
- `rpc-mem`: service trait expansion + read method registration + request/response schema.
- `whirlpool-node`: service wiring to provide finalized personality storage adapter.
- tests: add/extend contract tests for found/not-found/invalid-input while preserving submit regression.

## Guardrails
- Preserve existing submit behavior and method contract.
- Do not read pending mempool state for get endpoint.
- Keep deterministic hex encoding/decoding for binary fields.
