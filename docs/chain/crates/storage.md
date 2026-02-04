# `storage`

**Purpose**: persistence APIs for blocks + state.

Owns: block store, state DB interface, snapshots/pruning boundaries.

Inputs: blocks, receipts, state updates.

Outputs: reads for executor (state), consensus (history), RPC (queries).

Depends on: `types`.

Not in scope: indexers/derived views (optional separate component).
