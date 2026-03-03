| Design concept | File path | Function / struct | Status |
| --- | --- | --- | --- |
| InMemoryTxPool implementation (struct, new, push, TxSource::pending) | crates/app/src/traits.rs | `InMemoryTxPool`, `new()`, `push()`, `TxSource::pending()` | complete |
| App crate re-export of pool type | crates/app/src/lib.rs | `pub use traits::{..., InMemoryTxPool, ...}` | complete |
| Whirlpool node wiring of InMemoryTxPool into EvmApplication | crates/whirlpool-node/src/main.rs | `main` (creates `tx_pool` and passes to `EvmApplication::new`) | complete |
| Integration test `test_propose_with_in_memory_pool` | crates/app-evm/tests/integration.rs | `test_propose_with_in_memory_pool` | complete |
| Unit test `new_pool_is_empty` | crates/app/src/traits.rs | `fn new_pool_is_empty()` | complete |
| Unit test `push_single_tx` | crates/app/src/traits.rs | `fn push_single_tx()` | complete |
| Unit test `push_multiple_txs_fifo_order` | crates/app/src/traits.rs | `fn push_multiple_txs_fifo_order()` | complete |
| Unit test `pending_drains_buffer` | crates/app/src/traits.rs | `fn pending_drains_buffer()` | complete |
| Unit test `push_after_drain` | crates/app/src/traits.rs | `fn push_after_drain()` | complete |
| Unit test `concurrent_push` | crates/app/src/traits.rs | `fn concurrent_push()` | complete |
