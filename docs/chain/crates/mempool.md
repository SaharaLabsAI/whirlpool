# `mempool`

**Purpose**: transaction admission + pending-tx management.

Owns: tx validation (cheap checks), eviction rules, prioritization/order, proposer selection API.

Inputs: raw txs (from RPC/network).

Outputs: ordered tx batches for block building.

Depends on: `types` (tx format) + (optionally) `executor` for precheck hooks.
