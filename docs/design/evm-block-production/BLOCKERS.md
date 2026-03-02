# Blockers — EVM Block Production

## Active blockers

| ID | Type | Severity | Summary | Blocking | Evidence |
|---|---|---|---|---|---|
| BP-001 | implementation | Critical | `EvmApplication::propose()` only builds empty blocks — no transaction execution | INV-01, INV-06, INV-07, Success criteria #1 | `crates/app-evm/src/executor.rs::propose()` — returns block with empty tx list, no EVM invocation |
| BP-002 | implementation | Critical | `EvmApplication::verify()` only checks state_root match — does not re-execute transactions | INV-02, INV-03, Success criteria #2 | `crates/app-evm/src/executor.rs::verify()` — compares `state_root` only, no tx re-execution |
| BP-003 | implementation | Critical | Only `NoopTxSource` exists — no real transaction source for block production | Success criteria #3 | `crates/app/src/traits.rs::NoopTxSource` returns empty `Vec` |
| BP-004 | design-gap | High | Finalize→commit ownership path is UNKNOWN — unclear which component triggers `InMemoryStateDb::commit()` after consensus finalization | INV-05, Success criteria #4, #7 | `ConsensusEvent::Finalized` dispatched via `FinalizationSink` but no handler commits state |
| BP-005 | design-gap | Medium | State snapshot/rollback mechanism not defined — needed for INV-03 (verify read-only) and INV-04 (snapshot safety) | INV-03, INV-04, Success criteria #4 | `InMemoryStateDb` has no snapshot/restore API |
| BP-006 | scope-expansion | Low | MPT (Merkle Patricia Trie) needed for correct `transactions_root` and `receipts_root` computation | INV-06, Success criteria #5 | Currently tracked as B-003 in `docs/design/evm-integration/BLOCKERS.md` |
| BP-007 | scope-expansion | Low | State persistence / disk-backed storage needed for production use | Success criteria #4 (durability) | Currently tracked as B-004 in `docs/design/evm-integration/BLOCKERS.md` |

## Resolved blockers

_None yet — this is the initial design session._

## Blocker classification

- **implementation**: Missing code that the design specifies — must be built
- **design-gap**: Interface or ownership ambiguity that must be resolved before implementation
- **scope-expansion**: Dependency on out-of-scope capability — tracked but not blocking initial implementation
