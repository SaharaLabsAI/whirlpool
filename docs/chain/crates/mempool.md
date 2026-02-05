# `mempool`

**Purpose**: transaction admission + pending-tx management.

Owns: tx validation (cheap checks), eviction rules, prioritization/order, proposer selection API.

Inputs: raw **signed** tx bytes (from RPC/network).

Outputs: ordered signed tx batches for block building.

Depends on: `types` (e.g. `types::SignedTransaction`) + (optionally) `executor` for precheck hooks.
