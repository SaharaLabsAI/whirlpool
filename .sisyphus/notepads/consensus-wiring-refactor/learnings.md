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

## Refactor Summary (Completed)

**Goal Achieved**: Consensus wiring successfully moved from whirlpool-node to consensus-simplex.

**Files Moved and Genericized**:
1. mailbox.rs (351→394 lines) - MailboxActor<A: ConsensusApp>, Mailbox<B>
2. sink.rs (138→148 lines) - FinalizationSink<B: Block>

**Files Deleted from whirlpool-node**:
- mailbox.rs, sink.rs, wire.rs (558 lines total)
- never_enable_this feature gates removed

**Architecture Changes**:
- CommonwareEngine now uses constructor pattern: new(app, sink, config)
- Sealed internal wiring: Mailbox, MailboxActor, FinalizationSink created in start()
- EmptyBlockApp and EmptyBlock preserved unchanged (business logic)

**Commits Made**:
- b4bb886: Move and genericize Mailbox and FinalizationSink
- e91e98e: Replace starter closure with sealed engine wiring
- 98a4a3e: Update whirlpool-node to consume consensus-simplex API
- 6ad19f0: Clean up dependencies

**Zero EmptyBlock references in consensus-simplex** ✓
