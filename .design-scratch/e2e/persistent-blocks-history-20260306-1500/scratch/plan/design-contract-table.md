# Design Contract Table

Extracted from:
- `INTENT.md`, `STRATEGY.md`, `CRATES.md`, `FLOWS.md`, `DOMAINS.md`, `TESTS.md`, `BLOCKERS.md`, `WORKSPACE.md`
- `docs/crates/*.md`
- `persistent-blocks-history/proven-ac.md`

Status semantics:
- `gap` = requires implementation for this feature scope
- `existing` = already present baseline behavior or explicitly deferred/resolved (no MVP implementation task)

| ID | Type | Crate | Description | Design Source | Status |
|---|---|---|---|---|---|
| DC-001 | flow | consensus-simplex | Finalization activity emitted and digest-resolved from ephemeral `BlockStore` before sink forwarding | `FLOWS.md` Flow 1 steps 1-3; `crates/consensus-simplex.md` | existing |
| DC-002 | flow | app-evm -> state-reth | Finalized block persistence flow (`Finalized` event -> receipts recovery -> `store_block`) | `FLOWS.md` Flow 1 steps 5-10; `STRATEGY.md` Stream 2 | gap |
| DC-003 | flow | rpc-eth -> state-reth | `eth_getBlockByNumber` flow including tag resolution, storage read, conversion, and response | `FLOWS.md` Flow 2; `STRATEGY.md` Stream 3 | gap |
| DC-004 | flow | rpc-eth -> state-reth | `eth_getBlockByHash` flow via `HeaderNumbers` reverse lookup then number path | `FLOWS.md` Flow 3; `STRATEGY.md` Stream 3 | gap |
| DC-005 | flow | whirlpool-node | Node startup wiring flow extended with persistent block storage path | `FLOWS.md` Flow 4; `WORKSPACE.md` Integration Point 4 | gap |
| DC-006 | guardrail | state-reth | INV-1 MDBX atomicity: single write transaction persists all block artifacts or none | `proven-ac.md` INV-1; `TESTS.md` TC-SR-01/07 | existing |
| DC-007 | guardrail | state-reth | INV-2 TxNumber monotonicity across blocks via `BlockBodyIndices` continuity | `proven-ac.md` INV-2; `STRATEGY.md` Tx numbering decision | existing |
| DC-008 | guardrail | state-reth, rpc-eth | INV-3/INV-8 reconstruction + RPC type fidelity for block query correctness | `proven-ac.md` INV-3/8; `TESTS.md` TC-SR-03, TC-RPC-02/04 | existing |
| DC-009 | guardrail | app-evm, state-reth | INV-5 receipt-count invariant between decoded txs and receipts persisted | `proven-ac.md` INV-5; `FLOWS.md` Flow 1 step 6 | existing |
| DC-010 | guardrail | whirlpool-node | INV-7 consensus independence: persistence must not change generic consensus behavior | `proven-ac.md` INV-7; `STRATEGY.md` key decision #2 | existing |
| DC-011 | blocker | rpc-eth | BLK-3 deferred: persistent fallback for `eth_getTransactionReceipt` is post-MVP | `BLOCKERS.md` BLK-3 | existing |
| DC-012 | blocker | app-evm, state-reth | BLK-11 deferred: no retry policy on MDBX write failure; log and continue | `BLOCKERS.md` BLK-11 | existing |
| DC-013 | blocker | state-reth | BLK-8 resolved: EvmBlock/reth encoding mismatch handled via conversion functions | `BLOCKERS.md` BLK-8; `STRATEGY.md` decision #1 | existing |
| DC-014 | interface | app | Re-export `Receipt` for shared block-storage signature stability | `CRATES.md` app section; `crates/app.md` | gap |
| DC-015 | interface | state | Introduce `BlockStorage` trait (`store/get_by_number/get_by_hash/get_receipts`) and export in `lib.rs` | `STRATEGY.md` Stream 1; `crates/state.md` | gap |
| DC-016 | impl | app-evm | Make conversion visibility usable cross-crate and add receipt lifecycle (`pending_receipts`, `store_finalized_block`) | `STRATEGY.md` Stream 2; `crates/app-evm.md` | gap |
| DC-017 | impl | state-reth | Implement `BlockStorage` for `RethStateDb` with Headers/BodyIndices/Transactions/Receipts persistence + reads | `STRATEGY.md` Stream 1; `crates/state-reth.md` | gap |
| DC-018 | impl | rpc-eth | Add `eth_getBlockByNumber`/`eth_getBlockByHash`, BlockStorage-backed context/query path, and conversion handling | `STRATEGY.md` Stream 3; `crates/rpc-eth.md` | gap |
| DC-019 | wiring | whirlpool-node | Wire `PersistingFinalizationSink` and pass `RethStateDb` as block storage into RPC context | `FLOWS.md` Flow 4; `crates/whirlpool-node.md` | gap |
| DC-020 | test | state | Contract/object-safety checks for `BlockStorage` trait and bounds | `TESTS.md` TC-ST-01; `crates/state.md` test surface | gap |
| DC-021 | test | state-reth | Block storage unit suite (atomicity, number/hash round-trip, receipts, TxNumber continuity) | `TESTS.md` TC-SR-01..08; `crates/state-reth.md` | gap |
| DC-022 | test | app-evm | Receipt lifecycle + finalization persistence behavior tests | `TESTS.md` TC-AE-01..04; `crates/app-evm.md` | gap |
| DC-023 | test | rpc-eth | Endpoint tests for number/hash/tags/full-vs-hash response and conversion errors | `TESTS.md` TC-RPC-01..08; `crates/rpc-eth.md` | gap |
| DC-024 | test | integration-tests | End-to-end propose/finalize/store/query tests for SC-2/SC-5 | `TESTS.md` TC-INT-01/02, TC-FLW-01..04 | gap |
