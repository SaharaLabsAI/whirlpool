## [2026-02-25] Initial Context

### Architecture Overview
- 3-crate workspace: consensus (interface traits), consensus-simplex (commonware bridge), whirlpool-node (application)
- Goal: Move consensus wiring from whirlpool-node into consensus-simplex
- whirlpool-node should only contain business logic (EmptyBlock, EmptyBlockApp)

### Current State
- mailbox.rs and sink.rs in whirlpool-node are GENERIC infrastructure, not node-specific
- Both currently hardcode EmptyBlock type
- Task: genericize and move to consensus-simplex

### Test Pattern
- consensus-simplex/src/tests.rs has TestBlock, CollectorSink, MockApp
- Reuse these for migrated tests

### Build Environment Note
- cargo is at ~/.cargo/bin/cargo (not in PATH)
- C compiler (cc) missing - build verification may fail with linker errors (environment issue, not code issue)
