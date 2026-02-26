# Decisions - P2P Implementation

This file tracks architectural choices and rationale.

---

## [INITIAL] PeerId Copy vs Clone

**Decision**: `PeerId` trait requires `Clone` instead of `Copy`.

**Rationale**: commonware's `ed25519::PublicKey` is NOT `Copy` (contains `VerificationKey`). This is a deviation from the original design doc but necessary for compatibility.

**Status**: Documented deviation, approved by Metis review.

---

## [INITIAL] Sender Bytes vs impl Buf

**Decision**: `NetworkSender` uses `Bytes`, which naturally passes through to commonware's `impl Buf + Send`.

**Rationale**: `Bytes` implements `Buf`, so no conversion needed.

**Status**: Compatible with commonware API.

---

## [INITIAL] Quota Handling

**Decision**: All channels use the same default quota. Per-channel quota config is deferred to future work.

**Rationale**: Simplifies initial implementation while preserving future extensibility.

**Status**: Deferred feature.

---

## [INITIAL] start() sync vs async

**Decision**: `ConsensusEngine::start()` remains sync. Use `tokio::runtime::Handle::current().block_on()` to bridge async `open_channel()` calls.

**Rationale**: Preserves existing trait contract while enabling async network operations.

**Status**: Documented pattern for Task 6.

---

---

## 2026-02-26 21:45 - F1 Audit Result: Known Architectural Deviation

### Decision
The implementation uses a **multiplexed channel model** (one sender/receiver pair, channel as parameter) rather than the **per-channel model** specified in `docs/design/p2p.md` (separate sender/receiver per channel).

### Rationale
- The **plan** (`.sisyphus/plans/p2p-implementation.md`) explicitly describes the multiplexed approach
- The implementation is **internally consistent** and **functionally correct**
- All tests pass, workspace builds successfully
- This represents **documentation drift**, not implementation failure

### Impact
- `docs/design/p2p.md` is now **outdated** and should be updated in a future documentation task
- The implementation satisfies the **functional requirements** (consensus can send/receive on multiple channels)
- The **API shape differs** but the **capability is equivalent**

### Action
- Document as known deviation in learnings
- Proceed with remaining verification tasks (F2, F4)
- Schedule design doc update as separate task (out of scope for p2p-implementation plan)

