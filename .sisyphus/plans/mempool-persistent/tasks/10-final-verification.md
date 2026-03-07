# Task 10: Final Verification

<!-- TASK_META
wave: 8
depends_on: [09]
size: M
ac_refs: [AC-1, AC-2, AC-3, AC-4, AC-5, QA-1, QA-2, QA-3]
-->

## Objective

Run final verification of all acceptance criteria and invariants after implementation is complete.

## Pre-conditions

- All previous tasks (01-09) completed successfully
- `nix develop --command cargo build --workspace` exits 0
- `nix develop --command cargo test --workspace` exits 0

## Steps

### 1. Build Verification

```bash
nix develop --command cargo check --workspace 2>&1 | tee verification/cargo-check.log
nix develop --command cargo build --workspace 2>&1 | tee verification/cargo-build.log
nix develop --command cargo test --workspace 2>&1 | tee verification/cargo-test.log
nix develop --command cargo clippy --workspace --all-targets 2>&1 | tee verification/cargo-clippy.log
```

Log outputs to `.design-scratch/e2e/mempool-persistent-20260307-1000/verification/`.

### 2. AC Verification

Read `proven-ac.md` and verify each acceptance criterion:

| AC ID | Criterion | How to Verify |
|---|---|---|
| AC-1 | Transactions persist across restart | Run mempool persistence test (push, drop, reopen, verify pending) |
| AC-2 | Existing tests pass | cargo test --workspace shows 0 new failures |
| AC-3 | Mempool crate has unit tests | cargo test -p mempool shows test count > 0 |
| AC-4 | EthRpcContext works with trait object | cargo test -p rpc-eth passes |
| AC-5 | FIFO ordering verified | Run specific FIFO ordering tests |

| QA ID | Criterion | How to Verify |
|---|---|---|
| QA-1 | No clippy warnings | cargo clippy shows 0 warnings in modified crates |
| QA-2 | No unsafe code in mempool | grep for unsafe in crates/mempool/src/ |
| QA-3 | Error types propagated | Type system + integration tests pass |

### 3. INV Verification

| INV ID | Invariant | Test |
|---|---|---|
| INV-1 | FIFO ordering | Specific unit test |
| INV-2 | Drain semantics | pending() removes entries |
| INV-3 | Crash durability | Reopen test |
| INV-4 | Backward compatibility | InMemoryTxPool tests still pass |
| INV-5 | Thread safety | Concurrent test |
| XINV-1 | Cross-crate compat | Integration tests |
| XINV-2 | Build integrity | cargo build + test |

### 4. Write ac-verification.md

Write results to `.design-scratch/e2e/mempool-persistent-20260307-1000/verification/ac-verification.md`.

### 5. Report

Print summary with verdict and options:
- **Accept**: All AC/INV pass, proceed
- **Rollback**: `git reset --hard e2e-pre-execute-mempool-persistent-20260307-1000`
- **Fix**: Identify specific failures, create fix tasks
- **Abort**: Stop execution

## Post-conditions

- All AC items verified PASS or documented as FAIL with reason
- verification/ directory contains all logs
- ac-verification.md written with final verdict

## Must NOT

- Do NOT modify design documents
- Do NOT modify the plan files
- Do NOT skip any AC verification
- Build/test commands: `nix develop --command cargo build/test`
