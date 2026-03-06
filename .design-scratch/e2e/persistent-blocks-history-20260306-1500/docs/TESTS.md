# Test Contracts

## Strategy

The testing strategy for the "Persistent Block Storage & History Queries" feature focuses on three distinct layers:
1. **Storage Layer (state-reth)**: Unit tests with a temporary MDBX instance to verify the atomicity and round-trip fidelity of `BlockStorage` operations.
2. **Application Layer (app-evm)**: Tests for the receipt lifecycle and the new finalization hook, ensuring receipts are correctly cached during `propose` and flushed during `store_finalized_block`.
3. **RPC Layer (rpc-eth)**: Endpoint tests using mock storage to verify `eth_getBlockByNumber` and `eth_getBlockByHash` logic, including block tag resolution and full vs. hash response formatting.
4. **Integration/Wiring (whirlpool-node)**: End-to-end tests validating the full pipeline from block proposal to RPC retrieval.

Isolation is maintained via traits (`BlockStorage`, `StateDb`) allowing each layer to be tested independently with mocks or localized backends.

## Intent success-criteria mapping

| INTENT success criterion | Test section | Test case IDs |
|---|---|---|
| SC-1: Finalized blocks persisted to MDBX atomically | Unit Tests (state-reth) | TC-SR-01, TC-SR-07 |
| SC-2: Automatic persistence on finalization events | Integration Tests | TC-INT-01 |
| SC-3: `eth_getBlockByNumber` returns persisted data | Unit Tests (rpc-eth) | TC-RPC-02, TC-RPC-05, TC-RPC-07 |
| SC-4: `eth_getBlockByHash` returns persisted data | Unit Tests (rpc-eth) | TC-RPC-04 |
| SC-5: Node wiring integrates persistence + query | Integration Tests | TC-INT-02 |

## Unit Tests

### state
| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|---|---|---|---|---|---|---|---|
| BlockStorage | TC-ST-01 | Happy | N/A | Compile-time check | Verify `BlockStorage` is object-safe and trait bounds (`Send + Sync`) hold | High | [GROUNDED] |

### state-reth
| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|---|---|---|---|---|---|---|---|
| store_block | TC-SR-01 | Happy | Temp MDBX | Store block with 3 txs + 3 receipts | Single MDBX tx succeeds; all 8 tables populated correctly | High | [GROUNDED] |
| store_block | TC-SR-02 | Failure | Temp MDBX | Store block with mismatched receipt count | Return `RethStateError::Codec` or variant; MDBX transaction aborts | Medium | [PROPOSED] |
| get_block_by_number | TC-SR-03 | Happy | Persisted block | Call with existing number | `EvmBlock` reconstructed with 100% field fidelity | High | [GROUNDED] |
| get_block_by_number | TC-SR-04 | Happy | Empty DB | Call with number 999 | Return `Ok(None)` | Medium | [GROUNDED] |
| get_block_by_hash | TC-SR-05 | Happy | Persisted block | Call with block hash | `HeaderNumbers` lookup → number query succeeds; return `Some(EvmBlock)` | High | [GROUNDED] |
| get_block_by_hash | TC-SR-06 | Happy | Empty DB | Call with random hash | Return `Ok(None)` | Medium | [GROUNDED] |
| store_block | TC-SR-07 | Happy | Multiple blocks | Store blocks 1, 2, 3 sequentially | Verify `TxNumber` continuity in `BlockBodyIndices` (start = prev_end + 1) | High | [GROUNDED] |
| get_receipts_by_block | TC-SR-08 | Happy | Persisted block | Call with block number | Return exact receipts for that block in order | Medium | [GROUNDED] |

### app-evm
| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|---|---|---|---|---|---|---|---|
| propose | TC-AE-01 | Happy | EvmApplication instance | Call `propose(parent, height)` | `pending_receipts` contains non-None `Vec<Receipt>` with correct count | High | [GROUNDED] |
| store_finalized_block | TC-AE-02 | Happy | Mock DB + pending receipts | Call `store_finalized_block(block)` | `DB::store_block` called with receipts; `pending_receipts` becomes `None` | High | [GROUNDED] |
| store_finalized_block | TC-AE-03 | Failure | No pending receipts | Call `store_finalized_block(block)` | Call `store_block` with empty slice; return `Ok(())` (don't fail finalization) | Medium | [GROUNDED] |
| store_finalized_block | TC-AE-04 | Failure | DB write error | Call `store_finalized_block(block)` | Return `EvmAppError::State`; ensure error is logged but node doesn't crash | Medium | [PROPOSED] |

### rpc-eth
| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|---|---|---|---|---|---|---|---|
| get_block_by_number | TC-RPC-01 | Happy | Mock storage (empty) | Call with number 999 | Return JSON-RPC `null` | High | [GROUNDED] |
| get_block_by_number | TC-RPC-02 | Happy | Mock storage (1 block) | Call with number 0, `full=true` | Return full `alloy_rpc_types::Block` with all transactions | High | [GROUNDED] |
| get_block_by_number | TC-RPC-03 | Happy | Mock storage (1 block) | Call with number 0, `full=false` | Return block with transaction hashes only | Medium | [GROUNDED] |
| get_block_by_hash | TC-RPC-04 | Happy | Mock storage (1 block) | Call with hash | Return correct block or `null` if not found | High | [GROUNDED] |
| get_block_by_number | TC-RPC-05 | Happy | Context height = 10 | Call with "latest" or "finalized" | Resolve to number 10; query storage and return block | High | [GROUNDED] |
| get_block_by_number | TC-RPC-06 | Happy | N/A | Call with "pending" | Return JSON-RPC `null` (per MVP policy) | Medium | [GROUNDED] |
| get_block_by_number | TC-RPC-07 | Happy | Mock storage (1 block) | Call with "earliest" | Resolve to number 0; return block | Medium | [GROUNDED] |
| evm_block_to_rpc_block | TC-RPC-08 | Failure | Corrupt EvmBlock bytes | Internal conversion call | Return JSON-RPC internal error (-32000) with "tx decode error" | Low | [PROPOSED] |

### whirlpool-node
*Wiring logic only. Validated through integration tests.*

### consensus-simplex
| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|---|---|---|---|---|---|---|---|
| N/A | TC-CS-01 | Happy | N/A | No changes | Zero new tests for this crate as persistence is handled in Application layer | N/A | [GROUNDED] |

## Integration Tests

### Persistent Storage Domain
| Flow | Test case ID | Crates involved | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|---|---|---|---|---|---|---|---|---|
| Propose → Finalize → Store | TC-INT-01 | app-evm, state-reth | Temp MDBX | Propose block, then call finalization hook | `get_block_by_number` from DB returns exactly what was proposed | Real MDBX, real App | High | [PROPOSED] |
| Node Wiring | TC-INT-02 | whirlpool-node, rpc-eth, state-reth | Full Node | Start node, finalize block, call RPC | `eth_getBlockByNumber` via RPC returns valid block JSON | Real Full Stack | High | [PROPOSED] |

## Cross-Crate Flow Tests

| Flow | Test case ID | Entry -> Exit | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|---|---|---|---|---|---|---|---|---|
| Finalization → Storage | TC-FLW-01 | Simplex report → MDBX commit | Node + Simplex | Finalize block | Verify all 8 MDBX tables updated in one atomic transaction | Real Stack | High | [PROPOSED] |
| getBlockByNumber | TC-FLW-02 | RPC Request → Block JSON | RPC Context | `eth_getBlockByNumber` | Resolve tag → MDBX Read → Reconstruct → Convert → Response | Real Storage | High | [PROPOSED] |
| getBlockByHash | TC-FLW-03 | RPC Request → Block JSON | RPC Context | `eth_getBlockByHash` | `HeaderNumbers` Read → `get_block_by_number` path | Real Storage | High | [PROPOSED] |
| Node Startup | TC-FLW-04 | `main()` → RPC Ready | Config | Boot node | Verify `EthRpcContext` wired with same `RethStateDb` for state + storage | Real Stack | Medium | [PROPOSED] |

## Open Questions

| Gap / Unknown | Test case ID | Description | Impact | Label |
|---|---|---|---|---|
| Receipt timing edge case | TC-UNK-01 | `propose()` called but no finalization follows (stale receipts) | Potential for memory leak or mismatched persistence if not cleared | UNKNOWN |
| EvmBlock reconstruction fidelity | TC-UNK-02 | Round-trip `EvmBlock` → `Header` → `EvmBlock` loss | Verify unmapped fields (difficulty, extra_data) are handled consistently | UNKNOWN |
| MDBX write failure | TC-UNK-03 | `store_block` fails during MDBX write (e.g., disk full) | Ensure consensus doesn't halt and error is logged correctly | BLOCKER |
| Missing ephemeral block | TC-UNK-04 | `AppAdapter` can't find block in ephemeral store for finalization | Verify persistence is gracefully skipped and warning logged | UNKNOWN |
