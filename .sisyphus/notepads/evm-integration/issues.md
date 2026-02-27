
## [2026-02-28] Task 6 Prep - Dependency Fix Scope Creep
- **Issue**: Asked subagent to move alloy-trie from dev-dependencies to dependencies
- **Outcome**: Subagent correctly moved alloy-trie BUT also added alloy-consensus = "1.4.3" and alloy-eips = "1.4.3" (NOT requested)
- **Impact**: Harmless - cargo check/test/build all pass, likely made transitive deps explicit
- **Lesson**: Even simple "move one line" tasks can result in scope creep. Always verify with git diff.

## [2026-02-27T16:47Z] Task 6 gotcha
- The task's requested import path `app_evm::EvmApplication` is not exported from crate root; correct path is `app_evm::executor::EvmApplication`.

- Initial attempt using `revm = { path = "../../vendor/reth/crates/revm" }` failed: Cargo reported no matching package `revm` because that path package is named `reth-revm`.
- Initial attempt with `alloy-primitives = "0.8"` caused compile errors from mixed `alloy_primitives::B256` versions (`0.8.x` vs `1.5.x`) in `StateProvider` return type.

## Code Quality Issues (Task 10)

### Clippy Failures (-D warnings)

1. **state/src/db.rs:2** - Unused import `std::hash::BuildHasherDefault`
   - Must be removed before build passes with strict warnings

2. **app/src/adapter.rs:30** - Manual async fn pattern (clippy::manual-async-fn)
   - `propose` method should use `async fn` syntax instead of returning `impl Future`

3. **app/src/adapter.rs:43** - Manual async fn pattern (clippy::manual-async-fn)
   - `verify` method should use `async fn` syntax instead of returning `impl Future`

### Documentation Coverage

- **Overall coverage: 2.5%** (1 out of 40 public items documented)
- Only `StateProvider` trait has documentation
- All other public types, traits, functions, and modules lack doc comments
- This prevents generating useful rustdoc and makes the API harder to understand

### Priority Fixes

1. **CRITICAL:** Fix 3 clippy errors to pass `-D warnings`
2. **HIGH:** Add documentation to key public APIs (Application trait, InMemoryStateDb, EvmApplication)
3. **MEDIUM:** Document all remaining public items

