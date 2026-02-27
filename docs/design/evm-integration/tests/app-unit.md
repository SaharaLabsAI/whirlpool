# App Crate Unit Tests
> Unit tests for app crate interfaces.

Status: **PROPOSED** | Crate: `app`

## Block Construction

### `test_evm_block_trait_impl`
- **Layer**: unit
- **Crate**: `app`
- **Input**: `EvmBlock` with manual fields
- **Assert**: `id()` returns content hash, `height()` matches input, `parent_id()` matches input. **Grounded**: `crates/consensus/src/block.rs::Block` trait.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_evm_block_trait_impl() {
    let block = EvmBlock {
        height: 10, parent_id: [1; 32], state_root: [0; 32],
        transactions_root: [0; 32], receipts_root: [0; 32],
        gas_used: 0, timestamp: 0, transactions: vec![],
    };
    assert_eq!(block.height(), 10);
    assert_eq!(block.parent_id(), [1; 32]);
    assert!(block.id().iter().any(|&x| x != 0));
}

### `test_execution_result_fields`
- **Layer**: unit
- **Crate**: `app`
- **Input**: Manually created `ExecutionResult`
- **Assert**: All fields (state_root, gas_used, etc.) are correctly set. **[PROPOSED]**: `app::ExecutionResult` struct.
- **Pseudo-code**:
```rust
#[test]
fn test_execution_result_fields() {
    let res = ExecutionResult {
        state_root: [2; 32], receipts_root: [3; 32],
        gas_used: 100, receipt_count: 5,
    };
    assert_eq!(res.state_root, [2; 32]);
    assert_eq!(res.gas_used, 100);
    assert_eq!(res.receipt_count, 5);
}

## Adapter Forwarding

### `test_adapter_wrapping`
- **Layer**: unit
- **Crate**: `app`
- **Input**: `ApplicationAdapter` wrapping a `MockApplication`
- **Assert**: `inner()` returns reference to the mock. **[PROPOSED]**: `app::ApplicationAdapter::inner`.
- **Pseudo-code**:
```rust
#[test]
fn test_adapter_wrapping() {
    let mock = MockApplication::new();
    let adapter = ApplicationAdapter::new(mock);
    let _ = adapter.inner();
}

## Error Variants

### `test_application_error_display`
- **Layer**: unit
- **Crate**: `app`
- **Input**: `ApplicationError::Verification("fail".into())`
- **Assert**: Display output contains "fail". **[PROPOSED]**: `app::ApplicationError` variants.
- **Pseudo-code**:
```rust
#[test]
fn test_application_error_display() {
    let err = ApplicationError::Verification("fail".to_string());
    assert!(format!("{}", err).contains("fail"));
}
```

## Genesis Passthrough

### `test_adapter_genesis_passthrough`
- **Layer**: unit
- **Crate**: `app`
- **Input**: `ApplicationAdapter` wrapping a `MockApplication`
- **Assert**: `genesis()` delegates to inner `Application::genesis()`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::genesis`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_adapter_genesis_passthrough() {
    let expected_block = EvmBlock {
        height: 0, parent_id: [0; 32], state_root: [0; 32],
        transactions_root: [0; 32], receipts_root: [0; 32],
        gas_used: 0, timestamp: 0, transactions: vec![],
    };
    let mock = MockApplication::with_genesis(expected_block.clone());
    let adapter = ApplicationAdapter::new(mock);
    let genesis = adapter.genesis().await;
    assert_eq!(genesis.height(), 0);
    assert_eq!(genesis.parent_id(), [0; 32]);
}
```
