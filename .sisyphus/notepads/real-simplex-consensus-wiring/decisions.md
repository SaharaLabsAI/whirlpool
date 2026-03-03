## [2026-03-03] Task 03.3: Architectural Decisions

### Decision 1: Remove Type Wrappers from PerChannelNetwork

**Context**: Task 03.2 introduced `CommonwareSender<S>` and `CommonwareReceiver<R>` wrappers in `PerChannelNetwork`, but these wrappers do NOT implement vendor `commonware_p2p::{Sender, Receiver}` traits required by `simplex::Engine::start()`.

**Problem**: Type mismatch - vendor expects `impl commonware_p2p::Sender`, we provided `CommonwareSender<impl commonware_p2p::Sender>` which doesn't implement the trait.

**Options Considered**:
1. **Remove wrappers** (expose raw vendor types)
2. **Implement vendor traits on wrappers** (delegate all methods)

**Decision**: **Option 1 - Remove wrappers from PerChannelNetwork**

**Rationale**:
- The underlying vendor types already implement all required traits
- Wrappers were introduced without clear purpose and actively block integration
- Minimal changes required (2 struct definitions)
- Wrappers still exist for `MultiplexSender`/`MultiplexReceiver` which implement our vendor-agnostic `NetworkSender`/`NetworkReceiver` traits

**Implementation**:
```rust
// crates/p2p-commonware/src/provider.rs
pub struct PerChannelNetwork<S, R> {
    pub vote: (S, R),  // Raw vendor types
    pub cert: (S, R),
    pub resolver: (S, R),
    pub network_handle: commonware_runtime::Handle<()>,
}
```

**Impact**: Eliminated 6 compilation errors, unblocked Task 03.3 completion.

---

### Decision 2: Constrain Signer to ed25519::PublicKey

**Context**: Generic `C: Signer` with `C::PublicKey` caused type mismatch with Oracle/Blocker which expects concrete `ed25519::PublicKey`.

**Problem**:
```
error[E0271]: type mismatch resolving `<Oracle<<C as Signer>::PublicKey> as Blocker>::PublicKey == PublicKey`
    expected `PublicKey`, found associated type
    expected struct `commonware_cryptography::ed25519::PublicKey`
    found associated type `<C as commonware_cryptography::Signer>::PublicKey`
```

**Decision**: Add constraint `C: Signer<PublicKey = ed25519::PublicKey>`

**Rationale**:
- Design spec uses ed25519 throughout (Task 03.1-03.3, config.rs)
- Oracle is parameterized by `ed25519::PublicKey` specifically
- No other cryptographic schemes are currently supported or planned
- Aligns with vendor test patterns and examples

**Implementation**:
```rust
impl<A, S, E, C> ConsensusEngine for CommonwareEngine<A, S, E, C>
where
    // ...
    C: commonware_cryptography::Signer<PublicKey = ed25519::PublicKey> + Send + Sync + 'static,
```

**Impact**: Resolved Oracle/Blocker type mismatch, made signer type concrete.

---

### Decision 3: Pass Elector Config, Not Built Elector

**Context**: Initially called `RoundRobin::<Sha256>::default().build(&participants)` which returned `RoundRobinElector<Scheme>` but `simplex::Config` expects `L: Elector<S>` (the config type).

**Problem**:
```
error[E0277]: the trait bound `RoundRobinElector<_>: Config<Scheme>` is not satisfied
```

**Decision**: Pass `RoundRobin::<Sha256>::default()` (config) directly, not `.build()` result.

**Rationale**:
- `simplex::Config.elector` field is of type `L: Elector<S>` (the trait implementor)
- The `Elector::build()` method is called INTERNALLY by consensus engine, not by us
- `RoundRobin<H>` implements `Elector<S>`, not `RoundRobinElector<S>`
- Vendor patterns show passing config types, not built instances

**Implementation**:
```rust
let simplex_config = simplex::Config {
    scheme: scheme.clone(),
    elector: RoundRobin::<Sha256>::default(),  // Config, not .build() result
    // ...
};
```

**Impact**: Resolved elector type mismatch, reduced errors from 4 to 1.

---

### Decision 4: Use from_iter_dedup for Validator Set

**Context**: Need to convert `Vec<PublicKey>` to `Set<PublicKey>` for scheme construction.

**Options**:
1. `Set::try_from_iter()` - doesn't exist
2. `Set::from_iter_dedup()` - deduplicates and creates Set

**Decision**: Use `Set::from_iter_dedup()`

**Rationale**:
- `try_from_iter` doesn't exist in commonware-utils
- `from_iter_dedup` is the correct API per vendor documentation
- Deduplication is acceptable for validator sets (duplicates would be config error)

**Implementation**:
```rust
let participants = Set::from_iter_dedup(self.config.validators.clone());
```

---

### Decision 5: Direct Implementation After Subagent Timeouts

**Context**: Two subagent delegations for Task 03.3 resulted in 600s timeouts each (1200s total) with ZERO file changes or progress.

**Decision**: Proceed with direct orchestrator implementation as emergency measure.

**Rationale**:
- Systemic pattern of subagent failures on vendor integration tasks (6 timeouts total across Tasks 03.1-03.3)
- Task is blocking critical path (Tasks 03.4, 4, 5 depend on it)
- Orchestrator has full context from previous investigation
- Emergency provision in orchestrator protocol for critical blockers
- Task complexity is M (medium), within orchestrator capability

**Outcome**: Task 03.3 completed except for Task 4 blocker (AppAdapter Reporter), took ~1 hour vs 1200s wasted on timeouts.

**Lessons**:
- Vendor integration tasks may require direct orchestrator intervention
- Subagent delegation works well for isolated changes, struggles with multi-component vendor API wiring
- Emergency direct implementation saved ~2+ hours compared to continued delegation attempts

## [2026-03-03] Decision 6: Type Constraint Unification

**Context**: AppAdapter Reporter needed to use vendor's `Activity<Sig, Digest>` but our blocks use generic `Committable::Commitment` type.

**Decision**: Add explicit constraint `B: Committable<Commitment = Digest>` throughout:
- AppAdapter struct where clause
- All AppAdapter impl blocks
- Engine ConsensusEngine impl

**Rationale**:
- Vendor's `simplex::Engine` is hardcoded to `sha256::Digest` (not generic over D: Digest)
- Enforces compile-time type compatibility
- Simplifies HashMap key type (use `Digest` directly)
- Makes type relationships explicit and verifiable

**Alternatives Rejected**:
- Generic over D: Digest - vendor engine isn't generic, would require wrapper
- Runtime type conversion - error-prone, no compile-time safety
- Keep `<B as Committable>::Commitment` - adds unnecessary complexity when types must match anyway

**Impact**: Requires test block types (TestBlock) to use sha256::Digest as Commitment type (already the case).
