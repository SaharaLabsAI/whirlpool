# Task 03: Implement `eth_chainId` and `eth_gasPrice` (S2)

**Status**: [ ] pending
**Dependencies**: Task 02
**Wave**: 1
**Complexity**: S

## AC Coverage
- AC-1 (`eth_chainId` returns `313371`)
- AC-6 (`eth_gasPrice` returns `1_000_000_000` wei)

## Pre-Task Gate
- Confirm RPC module skeleton from Task 02 compiles.

## Context
Deliver low-risk constant-return methods first to validate endpoint wiring and type conversions.

## What to do
1. Implement `eth_chainId` in `eth_handler.rs` to return chain id value from `EthRpcContext` as RPC U64.
2. Implement `eth_gasPrice` in `eth_handler.rs` to return hardcoded `1 gwei` as U256 for v1.
3. Add targeted RPC tests for chain id and gas price behavior.
4. Ensure constants match proven AC values exactly.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/main/proven-ac.md` (AC-1, AC-6)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (S2)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/TESTS.md` (TC-001, TC-005)

## Acceptance Criteria
- `eth_chainId` returns `313371`.
- `eth_gasPrice` returns `1_000_000_000`.
- Corresponding tests exist and pass.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/03-chainid-gasprice.log`
