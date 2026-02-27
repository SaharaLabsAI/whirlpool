## [2026-02-27T16:47Z] Task 6 decision
- Kept production crates unchanged and implemented a test-local `TestStateDb` adapter to satisfy `EvmApplication<DB>`'s `DB: StateProvider` bound while still using `InMemoryStateDb` as underlying state storage.


## [2026-02-27T17:05Z] Task 7 decision
- Added a dedicated integration test target `crates/app-evm/tests/evm_execution_integration.rs` with three focused empty-block execution tests (execution metrics, state root equality, and propose->verify header-path validation) while leaving existing tests untouched.

## [2026-02-27T17:20Z] Task 8 decision
- Added `crates/app-evm/tests/cross_crate_flows.rs` as a standalone integration suite to validate app/app-evm/state interactions without modifying existing tests; reused local `TestStateDb` and `build_app()`/`build_adapter()` helpers for consistency.
