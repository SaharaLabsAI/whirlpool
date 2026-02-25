# Chain Binary Crate Scaffolding - Learnings

## Task 1: Crate Scaffold - COMPLETED

### Key Implementation Details

1. **Cargo.toml Structure**
   - Used workspace inheritance: `version.workspace = true`, `edition.workspace = true`
   - Dependencies mirrored directly from consensus-commonware crate
   - Added [[bin]] section with name and path for main.rs

2. **Module Organization**
   - Created 6 modules: config, block, app, sink, mailbox, wire
   - Module declarations in src/lib.rs with `pub mod` statements
   - Each module has stub implementation with TODO comment

3. **Config Module Hardcoding**
   - NAMESPACE: b"sahara-chain-v0" (byte literal)
   - BLOCK_INTERVAL: Duration::from_secs(5)
   - BIND_ADDR: "127.0.0.1:0" (wildcard port binding)
   - VALIDATOR_SEED: 0

4. **Workspace Registration**
   - Added "crates/chain-binary" to root Cargo.toml members list
   - Placement matters: must be in the members array structure

5. **Verification**
   - `cargo check -p chain-binary` passes cleanly
   - No blocking errors; only vendor deprecation warnings (expected)

### Process Notes

- File creation via write tool is atomic and straightforward
- Edit tool with LINE#ID references works well for precise array modifications
- Workspace member ordering doesn't affect build but should be consistent

### Next Task Dependencies

- All 6 modules ready for implementation in future tasks
- Config constants established and accessible
- Binary entry point prepared with placeholder main()

## TDD RED Phase - Task 2 EmptyBlock (2026-02-25)

### Test Suite Created
8 tests written covering:
1. Genesis block height = 0
2. Genesis parent = [0; 32]
3. Genesis ID determinism
4. Child height increments
5. Child-parent linking
6. Codec roundtrip
7. Digest determinism
8. Different blocks → different digests

### RED Phase Results
**Compilation failure** (expected):
- 13 errors: `use of undeclared type EmptyBlock`
- All tests reference EmptyBlock::{genesis, new, id, parent_id, height}
- Codec tests reference CodecWrite/CodecRead traits
- Digest tests reference Digestible trait

**Status**: RED phase confirmed ✓ — No implementation exists, tests fail at compile-time.


## TDD GREEN Phase - Task 2 EmptyBlock Implementation (2026-02-25)

### EmptyBlock Structure
```rust
pub struct EmptyBlock {
    height: u64,
    parent_id: [u8; 32],
}
```

### Dual-Trait Conformance Pattern
Successfully implemented BOTH trait hierarchies:

**consensus_core::Block** (our interface):
- `id() -> [u8; 32]` — computed via SHA-256(height || parent_id)
- `parent_id() -> [u8; 32]` — direct field access
- `height() -> u64` — direct field access

**Vendor traits** (commonware):
- **Codec**: `CodecWrite`, `CodecRead`, `EncodeSize` — binary serialization (8 bytes height + 32 bytes parent)
- **Heightable**: `height() -> Height` — wraps u64 in `Height::new(u64)`
- **Digestible**: `digest() -> Digest` — converts computed ID to vendor Digest type
- **Committable**: `commitment() -> Commitment` — delegates to digest()

### Method Conflict Resolution (CRITICAL GOTCHA)
Both `CoreBlock` and `Heightable` define `height()` with different return types:
- `CoreBlock::height() -> u64`
- `Heightable::height() -> Height`

**Solution**: Explicit trait qualification in test assertions:
```rust
assert_eq!(CoreBlock::height(&block), 5);  // Use fully qualified syntax
```

**Why this works**: Rust disambiguates based on trait bounds in generic contexts. For direct calls, explicit qualification is required.

### Implementation Details
1. **ID Computation**: SHA-256 hash of `height (8 bytes LE) || parent_id (32 bytes)` → deterministic 32-byte ID
2. **Genesis Constructor**: `height: 0, parent_id: [0u8; 32]`
3. **Codec Format**: Little-endian u64 followed by raw 32-byte parent ID (40 bytes total)
4. **Digest Mapping**: Uses `BlockDigest::from([u8; 32])` to convert computed ID to vendor digest type

### Test Results - GREEN Phase
All 8 tests PASS in 0.007s:
- Genesis height = 0 ✓
- Genesis parent = [0; 32] ✓
- Genesis ID determinism ✓
- Child height increments ✓
- Child-parent linking ✓
- Codec roundtrip ✓
- Digest determinism ✓
- Different blocks → different digests ✓

### Verification Status
- `cargo nextest run -p chain-binary block::tests` → 8/8 PASS ✓
- `cargo check -p chain-binary --lib` → Clean compilation ✓
- Clippy: Vendor p2p issues (not chain-binary) — our code is clean ✓

### Key Patterns for Future Tasks
1. **Trait conflict resolution**: Use `TraitName::method(&value)` when multiple traits define same method
2. **Vendor digest types**: Use `From<[u8; 32]>` trait to convert between our ID type and vendor digest
3. **Test-first design**: Writing tests FIRST forced clear API design before implementation
4. **Reference implementation**: TestBlock in consensus-commonware crate is reliable pattern source

### Architecture Notes
EmptyBlock is MINIMAL — no state, no transaction data. This is by design:
- Focus is on proving dual-trait conformance pattern works
- Real block types will extend this pattern with application-specific data
- This establishes the bridge between our `consensus_core::Block` interface and vendor consensus traits

