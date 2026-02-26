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

## [2026-02-26] Task 6 - LLMDocs Update Complete

### Documentation Updates
- Updated simplex-adapter.md: Added Mailbox, MailboxActor, FinalizationSink sections; documented sealed wiring approach
- Updated whirlpool-node.md: Removed old cfg gating references; documented that Mailbox/Sink moved to consensus-simplex
- Rewrote wiring-simplex-adapter.md: New API section (no starter closure); internal wiring sealed in CommonwareEngine
- Updated whirlpool-node-components.md: New "Architecture Evolution" section explaining post-refactor design
- Updated index.md: New descriptions of sealed wiring, consensus-simplex as library vs whirlpool-node as pure business logic

### Key Architectural Insights
1. **Sealed Wiring Pattern**: CommonwareEngine constructor now owns full component lifecycle (Mailbox↔MailboxActor, AppAdapter, FinalizationSink, simplex engine)
2. **Generic Types**: Mailbox<B>, MailboxActor<A>, FinalizationSink<B> all fully generic with zero EmptyBlock references in consensus-simplex
3. **Node Simplification**: whirlpool-node reduced to 3 core modules (block, app, config) + main; 19 tests total; pure business logic only
4. **Library vs Binary**: consensus-simplex is now a reusable library; whirlpool-node is a thin binary layer consuming it
5. **Documentation Principles**: Used fewest words necessary, based on actual code (no guesses), LLM-friendly for agents

### Files Updated
1. llmdocs/architecture/simplex-adapter.md (lines 7, 54-78, 94, 106-114)
2. llmdocs/architecture/whirlpool-node.md (completely rewritten)
3. llmdocs/guides/wiring-simplex-adapter.md (completely rewritten)
4. llmdocs/guides/whirlpool-node-components.md (updated with new architecture)
5. llmdocs/index.md (4 description lines updated)

### Verification
- All 5 doc files updated and saved
- Evidence saved: .sisyphus/evidence/task-6-llmdocs.txt
- grep confirms: sealed, Mailbox, FinalizationSink, generic types documented
- No EmptyBlock references appear in consensus-simplex architecture docs
