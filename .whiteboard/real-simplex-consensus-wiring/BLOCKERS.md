# BLOCKERS — Real Simplex Consensus Wiring

## Active Blockers

None.

## Information Gaps (UNKNOWN — non-blocking)

| # | Question | Searched | Impact | Default Assumption |
|---|----------|----------|--------|--------------------|
| 1 | Does simplex work correctly with n=1 validator? | Vendor test code uses n≥3 validators | Low — dev mode only | Assume yes; test empirically. If not, use n=1 with self-voting. |
| 2 | Does commonware tokio runtime context implement Storage trait? | `vendor/commonware/runtime/src/tokio/mod.rs` | Low — simplex may not require persistent storage | Assume in-memory storage is sufficient; use `commonware_storage::memory` if needed |
| 3 | What `buffer_pool` (PoolRef) value is appropriate? | Vendor test defaults | Low — performance tuning | Use vendor test defaults or `PoolRef::default()` |
| 4 | Exact generic type parameters for simplex::Engine with ed25519 + our types | Vendor engine.rs generics | Medium — complex generics | Derive from vendor test patterns; may require type aliases |

## Resolved Blockers

None (first iteration).
