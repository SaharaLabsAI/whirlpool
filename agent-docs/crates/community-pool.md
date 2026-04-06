# community-pool

## Purpose
Single-purpose crate that exposes the hardcoded on-chain community-pool account used by the current fee-accounting slice.

## Public API
- `COMMUNITY_POOL_ADDRESS: Address` — fixed EVM address credited with each block's burned amount in the current implementation.

## Status
Active but intentionally minimal. This crate is a narrow constant holder for the test-first community-pool flow and can be expanded later if reward policy/configuration becomes more sophisticated.
