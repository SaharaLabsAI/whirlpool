# Shared Crate Index — evmblock-txsource

| Crate | Role | Change Scope |
|---|---|---|
| `app` | Trait definitions (`TxSource`, `NoopTxSource`) | **Extend** — add `InMemoryTxPool` struct implementing `TxSource` |
| `app-evm` | EVM execution engine | **No changes** — already consumes `dyn TxSource` |
| `whirlpool-node` | Node binary / wiring | **Update** — replace `NoopTxSource` with `InMemoryTxPool` |

## Key Types

- `app::TxSource` — trait, `fn pending(&self) -> Vec<Vec<u8>>`
- `app::NoopTxSource` — stub impl
- `app-evm::EvmApplication<DB>` — consumer via `Arc<dyn TxSource + Send + Sync>`
- **[PROPOSED]** `app::InMemoryTxPool` — new struct implementing TxSource
