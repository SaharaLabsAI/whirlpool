
## [2026-02-28] Task 6 Prep - Dependency Fix Scope Creep
- **Issue**: Asked subagent to move alloy-trie from dev-dependencies to dependencies
- **Outcome**: Subagent correctly moved alloy-trie BUT also added alloy-consensus = "1.4.3" and alloy-eips = "1.4.3" (NOT requested)
- **Impact**: Harmless - cargo check/test/build all pass, likely made transitive deps explicit
- **Lesson**: Even simple "move one line" tasks can result in scope creep. Always verify with git diff.

## [2026-02-27T16:47Z] Task 6 gotcha
- The task's requested import path `app_evm::EvmApplication` is not exported from crate root; correct path is `app_evm::executor::EvmApplication`.
