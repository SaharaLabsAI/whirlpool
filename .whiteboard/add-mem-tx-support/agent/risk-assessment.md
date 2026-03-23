# Risk Assessment

## Accepted Risks
- R1: Prototype personality storage is in-memory only, so data is lost on restart and capacity is bounded only by process memory.
- R2: Signature verification remains structural-only in v1; cryptographic authenticity is deferred to a later Jolt-backed phase.
- R3: Replay/dedup policy may remain minimal at first, so later hardening may need additional storage keys or mempool rules.
- R4: Mixed transaction classification requires touching EVM-centric proposal/verification code, which raises regression risk unless design keeps EVM semantics isolated.

## Status
- No unresolved blocker prevents Design phase under the approved prototype scope.
