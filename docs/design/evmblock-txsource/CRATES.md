# CRATES — EvmBlock TxSource

| Crate | Role | Change Scope |
|---|---|---|
| `app` | Trait definitions + TxSource implementations | **Primary** — add `InMemoryTxPool` struct |
| `app-evm` | EVM execution engine | **Test only** — add integration test |
| `whirlpool-node` | Node binary | **Secondary** — update wiring |
