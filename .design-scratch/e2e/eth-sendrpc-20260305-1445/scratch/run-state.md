# Run State

## Step: 8 — Return verdict
## Sub-intent: Ethereum JSON-RPC server for balance transfers

## In-scope crates
| Crate | Role | Evidence | Status |
|-------|------|----------|--------|
| app | interface | `crates/app/src/traits.rs::Application`, `crates/app/src/traits.rs::TxSource` | grounded |
| whirlpool-node | node/binary | `crates/whirlpool-node/src/main.rs::main` | grounded |

## Interface crate API surface
| Interface Crate | Existing pub API | [PROPOSED] extensions | Consumers | Implementors |
|----------------|------------------|----------------------|-----------|--------------|
| app | `Application`, `TxSource`, `InMemoryTxPool`, `NoopTxSource`, `EvmBlock`, `ExecutionResult` (`crates/app/src/lib.rs`) | None for this design pass; RPC contracts remain node-local | `app-evm`, `whirlpool-node` | `app-evm::executor::EvmApplication` |

## Open blockers
| ID | Type | Summary | Evidence | Next question |
|----|------|---------|----------|---------------|
| — | — | None | — | — |

## Grounded vs Proposed tracking
| Item | Classification | Evidence/Rationale |
|------|---------------|-------------------|
| No JSON-RPC server exists today | grounded | `crates/whirlpool-node/src/main.rs::main` |
| RPC lifecycle insertion after `engine.start()` and before pending wait | grounded | `crates/whirlpool-node/src/main.rs::main` |
| Shared tx source handle is `Arc<InMemoryTxPool>` | grounded | `crates/whirlpool-node/src/main.rs::main`, `crates/app/src/tx_source.rs::InMemoryTxPool` |
| Chain id constant is `SAHARA_CHAIN_ID = 313_371` | grounded | `crates/app-evm/src/config.rs::SAHARA_CHAIN_ID` |
| Add node-local `eth` namespace via `jsonrpsee` 0.26 | [PROPOSED] | Matches reth vendor pattern (`vendor/reth/examples/node-custom-rpc/src/main.rs`) while preserving 3-layer separation |
| Provide minimal receipt synthesis from tracked tx metadata + nonce/finalized height | [PROPOSED] | Satisfies test-oriented scope without changing consensus traits |

## Digest pointers
- crate-index: `digests/shared-crate-index.digest.md`
- flows-index: `digests/shared-flows-index.digest.md`
- domain-map: `digests/shared-domain-map.digest.md`
- vendor-patterns: `digests/shared-vendor-patterns.digest.md`
- librarian: `digests/shared-librarian.digest.md`

## Completed steps
- Step 0: routing scan complete
- Step 1: intake gate complete (single sub-intent)
- Step 2: exploration synthesis complete
- Step 3: strategy/workspace synthesis complete
- Step 4: domains/wiring synthesis complete
- Step 5: hard blocker gate passed
- Step 6: build/finalize docs complete
- Step 7: sub-intent marked completed
- Step 8: verdict prepared (PASS)
