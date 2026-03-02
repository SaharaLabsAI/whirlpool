# Wiring

## Scope & method
- Wiring statements are restricted to evidence in shared and domains-wiring context plus crate index (`docs/design/evm-block-production/.design-scratch/run-20260301-1330/shared-context.md`; `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md`; `docs/design/evm-block-production/CRATES.md`).
- Reconciliation rule: shared-context key flows are treated as `INTENDED/PROPOSED` where grounded current wiring differs; current grounded behavior remains MVP-empty proposal/verification and lacks in-scope finalize->commit seam evidence (`docs/design/evm-block-production/.design-scratch/run-20260301-1330/shared-context.md::## Key flows`; `"MVP: Empty block execution (no transaction processing)"` in `crates/app-evm/src/executor.rs::EvmApplication::propose`; `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md::Confirmed blockers from code`).
- `UNKNOWN` marks missing proof in in-scope sources; `BLOCKER` marks intent-critical gaps already evidenced in code (`docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md::Explicit unknowns and blocker candidates`).

## Domain index
| Domain | Capabilities | File |
|---|---|---|
| Block Production | runtime bootstrap, network/engine startup, app injection into consensus | `wiring/block-production.md` |
| Application Layer | app lifecycle contract, tx ingress seam, consensus adapter mapping | `wiring/application-layer.md` |
| EVM Execution | genesis/propose/verify execution path, header conversion helpers, EVM config seam | `wiring/evm-execution.md` |
| State Management | database access traits, commit API, state-root derivation | `wiring/state-management.md` |

## Blockers
- `INV-01` (`Execution Visibility`) -> `BLOCKER`: non-empty tx path is not wired because runtime uses `NoopTxSource` and proposal emits empty tx list (`crates/whirlpool-node/src/main.rs::main`; `crates/app/src/traits.rs::NoopTxSource`; `crates/app-evm/src/executor.rs::EvmApplication::propose`; `docs/design/evm-block-production/tests/overview.md` INV-01).
- `INV-02` (`Verification Integrity`) -> `BLOCKER`: verify path does not recompute tx/receipt/gas artifacts, so tampered non-state-root fields are not proven checked (`crates/app-evm/src/executor.rs::EvmApplication::verify`; `docs/design/evm-block-production/tests/cross-crate-flows.md` "Mutate the `transactions_root` or `gas_used`" step; `docs/design/evm-block-production/tests/overview.md` INV-02).
- `INV-03` (`Verification Read-Only`) -> `UNKNOWN` for future full re-execution path; current code reads root under read lock, but no evidence yet for read-only guarantees once transaction replay is added (`crates/app-evm/src/executor.rs::EvmApplication::verify`; `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md` INV-03 note).
- `INV-04` (`Snapshot Safety`) -> `UNKNOWN/BLOCKER`: snapshot/rollback orchestration seam is unknown in in-scope node/app-evm wiring (`UNKNOWN`), and this is a blocker for intended non-empty execution safety (`BLOCKER`) even though DB exposes `Clone`/`commit` primitives (`crates/state/src/db.rs::InMemoryStateDb`; `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md` rollback UNKNOWN; `docs/design/evm-block-production/tests/overview.md` INV-04).
- `INV-05` (`Commit Atomicity`) -> `BLOCKER`: finalize-to-commit integration is not evidenced in in-scope app/consensus wiring (`crates/consensus/src/app.rs::ConsensusApp`; `crates/whirlpool-node/src/main.rs::main`; `docs/design/evm-block-production/tests/overview.md` INV-05).
- `INV-06` (`Root Consistency`) -> `BLOCKER`: propose path hardcodes empty roots with no tx execution, so roots are not derived from executed non-empty txs (`"MVP: Empty block execution (no transaction processing)" in `crates/app-evm/src/executor.rs::EvmApplication::propose`; `docs/design/evm-block-production/tests/overview.md` INV-06).
- `INV-07` (`Proposal Determinism`) -> `UNKNOWN` for real tx execution because ordering/selection policy is not defined by `TxSource` contract; current determinism is only for empty-block behavior (`crates/app/src/traits.rs::TxSource`; `crates/app-evm/src/executor.rs::EvmApplication::propose`; `docs/design/evm-block-production/tests/overview.md` INV-07).
