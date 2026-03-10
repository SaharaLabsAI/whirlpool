# BLOCKERS — Mempool Interface/Implementation Split

## Resolved

### BLK-001: MempoolError::Mdbx variant naming in interface crate

**Status:** RESOLVED  
**Severity:** Medium  
**Phase:** Identified in Step 2b (Convergence), resolved in Step 3 (Synthesis)

**Problem:** `MempoolError::Mdbx(String)` in the interface crate is storage-specific naming. An interface crate shouldn't reference a specific backend in its type names.

**Resolution:** Rename to `MempoolError::Storage(String)`. The variant already holds a `String` (no type dependency on MDBX). The `From<reth_libmdbx::Error>` impl moves out of the interface crate; `mempool-mdbx` constructs the error directly via `MempoolError::Storage(e.to_string())`.

**Migration Step:** Step 3 (Move store implementation).

## Open

None.
