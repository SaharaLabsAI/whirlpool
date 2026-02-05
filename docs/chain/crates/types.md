# `types`

**Purpose**: shared data model + cryptographic primitives used everywhere.

Owns: IDs/hashes/roots, block/tx structs, signatures, basic validation helpers, (de)serialization.

Consensus-adjacent (minimal): `Height`, `Round/View`, `ValidatorId`, `Vote`, `Finalization` (finalized block id + proof), and `Quorum`/threshold params.

Inputs/outputs: pure types + helper functions (no IO).

Depends on: minimal crypto + codec crates only.

Not in scope: networking, consensus rules, storage engines.
