## [2026-02-27T16:47Z] Task 6 decision
- Kept production crates unchanged and implemented a test-local `TestStateDb` adapter to satisfy `EvmApplication<DB>`'s `DB: StateProvider` bound while still using `InMemoryStateDb` as underlying state storage.

