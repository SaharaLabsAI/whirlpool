# Test Reference Grounding

## Existing Tests (16)

### store.rs Unit Tests
| TestID | Function Name | Line | Symbols Tested | Uses TempDir |
|---|---|---|---|---|
| TB-001 | test_push_and_drain | 85 | `MempoolStore` push/drain_pending round-trip | Yes |
| TB-002 | test_drain_empty | 98 | `MempoolStore` drain_pending empty result | Yes |
| TB-003 | test_drain_clears | 105 | `MempoolStore` drain_pending clears state | Yes |
| TB-004 | test_persistence_across_reopen | 118 | `MempoolStore` persistence across re-open | Yes |
| TB-005 | test_fifo_ordering | 134 | `MempoolStore` FIFO ordering of drain_pending | Yes |
| TB-006 | test_multiple_push_drain_cycles | 154 | `MempoolStore` repeated push/drain cycles | Yes |
| TB-007 | test_concurrent_push | 169 | `MempoolStore` concurrent push ordering + integrity | Yes |

### persistent.rs Unit Tests
| TestID | Function Name | Line | Symbols Tested | Uses TempDir |
|---|---|---|---|---|
| TB-008 | test_txsource_trait_object | 43 | `PersistentTxPool`, `TxSource` trait object coercion | Yes |
| TB-009 | test_pending_drains | 55 | `PersistentTxPool` pending/drain semantics via `TxSource` | Yes |
| TB-010 | test_persistence | 67 | `PersistentTxPool` persistence across reopen | Yes |

### Integration Tests
| TestID | Function Name | Line | Symbols Tested | Uses TempDir |
|---|---|---|---|---|
| TB-011 | trait_object_coercion_across_crates | 17 | `PersistentTxPool` + `TxSource` across crates | Yes |
| TB-012 | restart_recovery_via_trait | 36 | `PersistentTxPool` + `TxSource` persistence after restart | Yes |
| TB-013 | restart_after_drain_is_empty | 60 | `PersistentTxPool` drain durability | Yes |
| TB-014 | fifo_ordering_preserved | 83 | `PersistentTxPool` FIFO ordering through `TxSource` | Yes |
| TB-015 | fifo_ordering_survives_restart | 105 | `PersistentTxPool` FIFO across restart | Yes |
| TB-016 | concurrent_push_via_trait_object | 127 | `PersistentTxPool` concurrency via `TxSource` trait object | Yes |

## New Tests (2)
| TestID | Status | Description |
|---|---|---|
| TN-001 | [NEW] | Compile-time check that `MdbxMempoolStore` implements `MempoolStore` trait (not present yet) |
| TN-002 | [NEW] | Object-safety check for `MempoolStore` trait (not present yet) |

## Verification Result
- All TB-NNN found: YES (none missing)
- All TN-NNN confirmed new: YES
