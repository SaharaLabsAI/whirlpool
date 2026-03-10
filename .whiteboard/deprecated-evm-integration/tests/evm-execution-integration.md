# EVM Execution Integration Tests
> Integration tests for EVM execution domain.

Status: **PROPOSED** | Crate: `app-evm`

## Executor Wiring

### `test_executor_factory_wiring`
- **Layer**: integration
- **Crate**: `app-evm`
- **Input**: `WhirlpoolEvmConfig`
- **Assert**: Successfully creates an executor from the factory. **Grounded**: `alloy_evm::block::BlockExecutorFactory::create_executor` (re-exported via `vendor/reth/crates/evm/evm/src/execute.rs`).
- **Pseudo-code**:
```rust
#[test]
fn test_executor_factory_wiring() {
    let config = WhirlpoolEvmConfig::new(spec);
    let factory = config.block_executor_factory();
    let state = State::builder().with_bundle_update().build();
    let executor = factory.create_executor(state);
    // Executor created without panic — wiring is valid
}

## State Transitions

### `test_execute_empty_block`
- **Layer**: integration
- **Crate**: `app-evm`
- **Input**: Empty transactions list
- **Assert**: Execution succeeds with no state changes. **[PROPOSED]**: `app_evm::EvmApplication::propose`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_execute_empty_block() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    let (block, result) = app.propose(&genesis, 1).await.unwrap();
    assert_eq!(block.transactions.len(), 0);
    assert_eq!(result.receipt_count, 0);
}

## State Root Consistency

### `test_state_root_computation`
- **Layer**: integration
- **Crate**: `app-evm`
- **Input**: Simple transaction
- **Assert**: Propose then verify yields matching state roots. **[PROPOSED]**: `app_evm::EvmApplication::propose` + `verify`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_state_root_computation() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    let (block, result) = app.propose(&genesis, 1).await.unwrap();
    assert_eq!(block.state_root, result.state_root);
}

### `test_reconstruct_header_for_verify`
- **Layer**: integration
- **Crate**: `app-evm`
- **Input**: `EvmBlock` from `propose`
- **Assert**: Reconstructed header matches original block fields. **[PROPOSED]**: `app_evm::EvmApplication::verify` internal reconstruction.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_reconstruct_header_for_verify() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    let (block, _) = app.propose(&genesis, 1).await.unwrap();
    let result = app.verify(&genesis, &block).await.unwrap();
    assert_eq!(result.state_root, block.state_root);
}
```
