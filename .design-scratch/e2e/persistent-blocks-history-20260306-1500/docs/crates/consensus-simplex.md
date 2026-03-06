# consensus-simplex crate — Persistent Block Storage Contract

## Purpose

**Today**: Commonware Simplex BFT adapter. Provides `AppAdapter` (consensus callbacks), `MailboxActor` (message handling), `FinalizationSink` (event sink tracking finalized height), `CommonwareEngine` (lifecycle), and `BlockStore<B>` (ephemeral `Arc<RwLock<HashMap<Digest, B>>>`). All types are generic over `B: Block`.

**Changes**: **None required.** Block persistence happens at the application layer (`app-evm`), not the consensus layer. The generic `B: Block` constraint prevents `consensus-simplex` from knowing about concrete EVM types (`Header`, `TransactionSigned`, `Receipt`) or storage implementations (`BlockStorage`).

## Public API Changes

None. No types, traits, methods, or signatures change.

## Internal Changes

None. The finalization flow remains:

1. Simplex consensus finalizes block -> `Activity::Finalization`
2. `AppAdapter::report()` extracts `Digest`, looks up block in ephemeral `BlockStore<B>` (HashMap)
3. `AppAdapter` forwards `ConsensusEvent::Finalized { block, height, proof }` to `EventSink`
4. `FinalizationSink::handle()` updates `Arc<AtomicU64>` height counter
5. **NEW (in whirlpool-node, NOT here)**: The node-level finalization handler also calls `EvmApplication::store_finalized_block()` for persistence

The ephemeral `BlockStore<B>` (HashMap) continues to serve its existing purpose — it is NOT replaced by persistent `BlockStorage`. It holds proposed blocks temporarily between propose and finalization for the consensus protocol.

## Dependencies

No changes. Existing dependencies:

- `consensus = { path = "../consensus" }` — for `Block`, `EventSink`, `ConsensusEvent` traits
- `p2p = { path = "../p2p" }` — for `Channel` type
- `commonware-*` — for Simplex BFT protocol

## Error Types

No changes.

## Test Surface

No new tests needed. Existing tests remain valid:

- `test_handle_finalized_logs_height`
- `test_handle_finalized_updates_atomic_height`
- `test_handle_prefinalized_is_noop`
- `test_handle_fault_logs_warning`
- `test_height_monotonically_increases`
- `test_initial_height_is_zero`

## Integration Points

| Connected Crate | Direction | Interface | Impact |
|-----------------|-----------|-----------|--------|
| `consensus` | Depends on | `Block`, `EventSink`, `ConsensusEvent` traits | No change |
| `whirlpool-node` | Used by | `FinalizationSink`, `CommonwareEngine`, `AppAdapter` | No change — node adds separate persistence hook |

**Key architectural decision**: Persistence is at the application layer boundary, not the consensus layer. This preserves the generic `B: Block` constraint and keeps consensus-simplex reusable for non-EVM applications.

**Source**: STRATEGY.md Key Design Decision 2, CRATES.md consensus-simplex section ("None required"), DOMAINS.md Consensus/Finalization Domain
