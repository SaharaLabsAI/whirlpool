# Application Domain Integration Tests
> Integration tests for application domain.

Status: **PROPOSED** | Crate: `app`

## Adapter Flow

### `test_adapter_propose_success`
- **Layer**: integration
- **Crate**: `app`
- **Input**: `ApplicationAdapter` with successful `MockApplication`
- **Assert**: `propose` returns `Some(Block)`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::propose`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_adapter_propose_success() {
    let genesis = EvmBlock {
        height: 0, parent_id: [0; 32], state_root: [0; 32],
        transactions_root: [0; 32], receipts_root: [0; 32],
        gas_used: 0, timestamp: 0, transactions: vec![],
    };
    let mock = MockApplication::new_success(genesis.clone());
    let adapter = ApplicationAdapter::new(mock);
    let res = adapter.propose(&genesis, 1).await;
    assert!(res.is_some());
}

### `test_adapter_propose_failure`
- **Layer**: integration
- **Crate**: `app`
- **Input**: `ApplicationAdapter` with failing `MockApplication`
- **Assert**: `propose` returns `None`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::propose`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_adapter_propose_failure() {
    let parent = make_test_block(0);
    let mock = MockApplication::new_failure(ApplicationError::Execution("fail".into()));
    let adapter = ApplicationAdapter::new(mock);
    let res = adapter.propose(&parent, 1).await;
    assert!(res.is_none());
}

### `test_adapter_verify_success`
- **Layer**: integration
- **Crate**: `app`
- **Input**: `ApplicationAdapter` with successful `MockApplication`
- **Assert**: `verify` returns `Ok(())`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::verify`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_adapter_verify_success() {
    let (parent, block) = make_test_block_pair();
    let mock = MockApplication::new_verify_success();
    let adapter = ApplicationAdapter::new(mock);
    let res = adapter.verify(&parent, &block).await;
    assert!(res.is_ok());
}

### `test_adapter_verify_failure`
- **Layer**: integration
- **Crate**: `app`
- **Input**: `ApplicationAdapter` with failing `MockApplication`
- **Assert**: `verify` returns `Err(ConsensusError::Verification)`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::verify`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_adapter_verify_failure() {
    let (parent, block) = make_test_block_pair();
    let mock = MockApplication::new_failure(ApplicationError::Verification("fail".into()));
    let adapter = ApplicationAdapter::new(mock);
    let res = adapter.verify(&parent, &block).await;
    assert!(matches!(res, Err(ConsensusError::Verification(_))));
}
```
