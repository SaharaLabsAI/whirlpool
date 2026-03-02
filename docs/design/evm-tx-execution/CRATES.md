# Crates — EVM Transaction Execution

| Crate | Path | Purpose | Changes |
|---|---|---|---|
| `app-evm` | `crates/app-evm` | EVM configuration and block execution wrapper | Replace propose/verify stubs with real EVM execution via reth BlockBuilder/BlockExecutor |
| `state` | `crates/state` | In-memory EVM state database | Verify commit() handles all BundleState fields; potential minor fixes |
| `app` | `crates/app` | Application traits and types (EvmBlock, TxSource) | **No changes** — stable trait definitions consumed by app-evm |
| `consensus` | `crates/consensus` | Consensus traits (ConsensusApp, Block) | **No changes** — out of scope |
| `whirlpool-node` | `crates/whirlpool-node` | Node binary wiring | **No changes** — deferred to Sub-Intent 3 |
