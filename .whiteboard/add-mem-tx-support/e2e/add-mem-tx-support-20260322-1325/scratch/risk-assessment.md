# Risk Assessment

## Iteration
- alignment_iteration: 1

## Resolved Risks
- None in alignment phase; current work only confirmed that the existing mempool boundary already carries opaque bytes.

## Accepted Risks
- R1: Prototype personality storage is in-memory only, so data is lost on restart and capacity is bounded only by process memory.
- R2: Signature verification remains structural-only in v1; cryptographic authenticity is deferred to a later Jolt-backed phase.
- R3: Replay/dedup policy may remain minimal at first, so later hardening may need additional storage keys or mempool rules.
- R4: Mixed transaction classification requires touching EVM-centric proposal/verification code, which raises regression risk unless design keeps EVM semantics isolated.

## Blocker Conversions
- None currently. No unresolved blocker prevents Design phase, assuming the user accepts the v1 prototype tradeoffs.

## Expansion Summary
- No scope expansion proposed during alignment. Submit-only mem RPC, generic mempool reuse, and prototype in-memory storage remain in scope.
