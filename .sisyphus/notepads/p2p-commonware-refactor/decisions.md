# Decisions - p2p-commonware-refactor

## [2026-02-26T14:45:04.414Z] Session Start
- Builder returns `(CommonwareNetworkProvider, OracleHandle)` tuple since `start()` consumes provider
- `OracleHandle` wraps cloned Oracle for post-build validator updates
- Bootstrapper type: `Vec<Bootstrapper<C::PublicKey>>` re-exported from p2p-commonware
- Builder uses `Config::local()` only (production builder out of scope)
- Keep existing `new()` and `with_config()` constructors as low-level escape hatches
