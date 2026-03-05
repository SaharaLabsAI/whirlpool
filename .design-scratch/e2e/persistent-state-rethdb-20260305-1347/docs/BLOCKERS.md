# BLOCKERS

The following blockers are triaged from `STRATEGY.md` against `INTENT.md` and classified by impact on downstream design phases (crate contracts and domain wiring).

| ID | Description | Severity | Resolution strategy | Owner |
|---|---|---|---|---|
| BLK-001 | `StateDb` fallibility migration is not yet locked as a cross-crate contract (`state`, `state-memory`, `app-evm`, `rpc-eth`, `state-reth`). Without final signature/bounds agreement, crate contracts cannot be finalized. | hard | Approve a single canonical `StateDb` signature set (associated `Error` + `Result` returns), plus required trait bounds and migration expectations for all consumers. | design |
| BLK-002 | Trie-root design is directionally chosen (`reth_trie::StateRoot::overlay_root`), but the contract for hashed-state construction inputs/coverage is not yet pinned. This blocks precise domain wiring for `trie.rs` and acceptance criteria. | hard | Define a concrete state-root contract: exact source tables, hashing/normalization rules, and correctness oracle (test vectors / fixtures) used for validation. | design |
| BLK-003 | MDBX/reth-db host prerequisites are unspecified (native build/runtime requirements and environment assumptions). This is a hard execution blocker for persistent backend adoption. | hard | Add an explicit prerequisites contract for build/run environments (required toolchain/packages, platform assumptions, and failure policy when prerequisites are missing). | design |
| BLK-101 | Block-hash persistence mapping is still TBD (`CanonicalHeaders` vs `HeaderNumbers`) for `get_block_hash` / `insert_block_hash`. | soft | Choose final table mapping during implementation after API confirmation; keep `StateDb` method contract stable meanwhile. | implementation |
| BLK-102 | Final `RethStateError` variant taxonomy and exact conversion mapping are not finalized. | soft | Refine error enum during implementation while preserving top-level categories (`Database`, `Codec`, `StateRoot`, `Init`) and consumer-facing mapping behavior. | implementation |
| BLK-103 | Internal performance strategy (transaction batching/caching) is unspecified. | soft | Start with correctness-first per-method transactions; add profiling-guided caching/batching in implementation without changing external contracts. | implementation |
