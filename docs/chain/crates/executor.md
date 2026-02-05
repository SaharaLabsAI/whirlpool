# `executor`

**Purpose**: deterministic state transition.

Owns: execute block/tx list -> new state, receipts, `state_root`.

Inputs: parent state + ordered **signed** txs.

Outputs: post-state (or state diff), receipts, roots.

Depends on: `types`, `storage` (state access).

Boundary: **executor != consensus** (no voting/finality here).
