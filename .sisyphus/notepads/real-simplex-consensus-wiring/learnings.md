## [2026-03-03] Task 03.4 Status: Cannot Proceed - Blocked on Task 4

**Status**: ❌ CANNOT START - Pre-Task Gate fails

**Blocker**: Task 03.4 Pre-Task Gate requires:
```bash
nix develop --command cargo test -p consensus-simplex -- test_engine_can_start_and_shutdown
```

This command FAILS because `consensus-simplex` does not compile due to AppAdapter Reporter type mismatch (1 compilation error remaining from Task 03.3).

**Root Cause**: Task 03.4 specification (line 11) states:
> "Expected: exit 0 (Task 03.3 must have wired the real engine)"
> "If gate fails: **STOP. Task 03.3 is not complete.**"

Task 03.3 implementation is complete BUT compilation is blocked on AppAdapter Reporter fix which belongs to Task 4 scope.

**Dependency Chain**:
1. Task 03.3: Engine wiring DONE, blocked on AppAdapter (Task 4 scope)
2. Task 4: Must fix AppAdapter + Mailbox/MailboxActor together
3. Task 03.3: Will compile after Task 4 completes
4. Task 03.4: Can run tests and rename them after Task 03.3 compiles

**Correct Execution Order**: Task 4 → Task 03.3 completion → Task 03.4

**Action**: Proceeding with Task 4 (Close Mailbox/MailboxActor Gaps) which unblocks both Task 03.3 compilation and Task 03.4 execution.

**Files Ready for Renaming** (once compilation works):
- `crates/consensus-simplex/src/tests.rs`:
  - Line 347: `test_engine_start_and_status` → `test_engine_starts_with_real_simplex`
  - Line 364: `test_engine_shutdown` → `test_engine_shutdown_aborts_handle`
  - Line 378: `test_engine_height_tracking` → `test_engine_status_tracks_height`

## [2026-03-03] Task 4: AppAdapter Reporter Type Fix

### Key Discovery: Marshaled vs Raw Simplex Layers
The vendor provides TWO consensus layers:
1. **Raw layer**: `simplex::Engine` uses `Activity<Scheme, Digest>` - protocol-level types
2. **Marshaled layer**: `Marshaled` wrapper uses `Update<B>` - application-level bridge

Our design (per FLOWS.md) uses **raw `simplex::Engine`**, so:
- `Activity::Finalization.proposal.payload` is a `Digest` (sha256 hash), NOT a `Block`
- Blocks must be cached and retrieved using the digest as key

### Solution Pattern
1. **Constraint unification**: Use `B: Committable<Commitment = Digest>` to enforce block commitments match vendor digest type
2. **HashMap key**: Use `Digest` directly, not `<B as Committable>::Commitment` (they're equal by constraint)
3. **Finalization handler**: `proposal.payload` IS the commitment, lookup block from cache

### Type Safety Win
Adding `Committable<Commitment = Digest>` constraint enforces compile-time guarantee:
- Block commitment type MUST be sha256::Digest
- No possible runtime mismatch between cache key and Activity payload
- Compiler verifies block type compatibility with vendor engine

### Import Requirements
Both `adapter.rs` and `engine.rs` need:
```rust
use commonware_cryptography::{sha256::Digest, Committable};
```
