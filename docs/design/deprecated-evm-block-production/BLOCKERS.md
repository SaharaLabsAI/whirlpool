# Blockers

## Summary
- Total: 7 blockers (2 scope-expansion, 4 information-gap, 1 decision-gap)
- Resolved this run: 1

## Active blockers

### [B-001] `propose()` still produces empty blocks (core)
- **Type**: `information-gap`
- **Severity**: `blocking`
- **Affected docs**: `INTENT.md`, `architecture/block-proposal.md`, `app-evm/README.md`, `tests/evm-execution-integration.md`
- **Description**: The current proposal path does not execute transactions, so success criteria for non-empty execution are not met.
- **Attempted action** (information-gap): Verified current design evidence and test contracts; no contradictory docs found.
- **Suggested resolution**: Specify and implement the full `propose()` execution path (tx ingestion, EVM execution, result assembly).

### [B-002] `verify()` does not re-execute transactions (core)
- **Type**: `information-gap`
- **Severity**: `blocking`
- **Affected docs**: `INTENT.md`, `architecture/block-verification.md`, `app-evm/README.md`, `tests/app-evm-unit.md`
- **Description**: Verification currently compares roots rather than replaying transactions, leaving verification-integrity criteria unmet.
- **Attempted action** (information-gap): Cross-checked architecture and tests; replay contract is still proposed/blocked.
- **Suggested resolution**: Define and implement deterministic replay in `verify()` with mismatch rejection semantics.

### [B-003] Only `NoopTxSource` is available (core)
- **Type**: `information-gap`
- **Severity**: `blocking`
- **Affected docs**: `INTENT.md`, `app/README.md`, `whirlpool-node/README.md`, `architecture/node-startup.md`
- **Description**: There is no concrete transaction source in runtime wiring, so proposals remain empty.
- **Attempted action** (information-gap): Reviewed crate contracts and startup architecture; concrete source is still proposed only.
- **Suggested resolution**: Add and wire a concrete `TxSource` implementation with deterministic ordering contract.

### [B-004] Finalize->commit ownership path is unknown
- **Type**: `decision-gap`
- **Severity**: `blocking`
- **Affected docs**: `architecture/state-commitment.md`, `architecture/block-finalization.md`, `wiring/block-production.md`, `state/README.md`
- **Description**: It remains undefined which component owns the trigger from consensus finalization to canonical state commit.
- **Suggested resolution**: Make an explicit ownership decision (component + callback seam + error handling contract), then propagate it through architecture/wiring/tests.

### [B-005] Snapshot/rollback orchestration seam is undefined
- **Type**: `information-gap`
- **Severity**: `degraded`
- **Affected docs**: `architecture/overview.md`, `domains/state-management.md`, `wiring/state-management.md`, `tests/state-unit.md`
- **Description**: Snapshot primitives exist, but runtime orchestration for failure-safe execution boundaries is not fully specified.
- **Attempted action** (information-gap): One auto-resolution pass collected grounded INV-03 evidence; this seam remains partially unspecified for full replay path.
- **Suggested resolution**: Define snapshot boundary ownership and rollback behavior for propose/verify failure paths.

### [B-006] MPT-based state root and execution roots are out of scope
- **Type**: `scope-expansion`
- **Severity**: `degraded`
- **Affected docs**: `INTENT.md`, `architecture/block-proposal.md`, `state/README.md`
- **Description**: Correct trie-backed root computation is required for production fidelity but intentionally deferred.
- **Required interface** (scope-expansion only): `trait TrieRootProvider { fn transactions_root(&self, txs: &[Vec<u8>]) -> [u8; 32]; fn receipts_root(&self, receipts: &[u8]) -> [u8; 32]; fn state_root(&self) -> [u8; 32]; }`
- **Suggested resolution**: Track under `evm-integration` B-003 and integrate once trie backend is available.

### [B-007] Durable state persistence is out of scope
- **Type**: `scope-expansion`
- **Severity**: `degraded`
- **Affected docs**: `INTENT.md`, `state/README.md`, `architecture/state-commitment.md`
- **Description**: Current in-memory state is non-durable, which limits restart/recovery guarantees.
- **Required interface** (scope-expansion only): `trait DurableStateStore { fn load_latest(&self) -> Result<(), StoreError>; fn commit_block(&self, height: u64, state_root: [u8; 32]) -> Result<(), StoreError>; }`
- **Suggested resolution**: Track under `evm-integration` B-004 and add persistence integration in a later scope.

## Resolved blockers (this run)

### [B-R01] Unlabeled speculative INV-03 wording in domain docs
- **Resolution**: Auto-resolution pass confirmed grounded evidence in in-scope docs (`state/README.md`, `app-evm/README.md`, `tests/overview.md`) and reclassified the issue as resolved documentation-evidence alignment.
- **Affected docs**: `domains/block-production.md`, `domains/evm-execution.md`, `tests/overview.md`
