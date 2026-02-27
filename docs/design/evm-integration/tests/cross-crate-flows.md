# Cross-Crate Flow Tests
> End-to-end flow tests for EVM integration.

Status: **PROPOSED** | Crate: `app`, `app-evm`

## Propose-Verify Cycle

### `test_propose_verify_success`
- **Layer**: e2e
- **Crate**: `app`, `app-evm`
- **Input**: Genesis block
- **Assert**: Block from `propose` is accepted by `verify`. **[PROPOSED]**: Full flow through `app::ApplicationAdapter` + `app_evm::EvmApplication`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_propose_verify_success() {
    let app = EvmApplication::new(db, spec);
    let adapter = ApplicationAdapter::new(app);
    let genesis = adapter.genesis().await;
    let block = adapter.propose(&genesis, 1).await.unwrap();
    let res = adapter.verify(&genesis, &block).await;
    assert!(res.is_ok());
}

### `test_state_root_mismatch`
- **Layer**: e2e
- **Crate**: `app`, `app-evm`
- **Input**: Block with modified state root
- **Assert**: `verify` fails with state root mismatch error. **[PROPOSED]**: `app_evm::EvmAppError::StateRootMismatch`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_state_root_mismatch() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    let (mut block, _) = app.propose(&genesis, 1).await.unwrap();
    block.state_root = [0xff; 32];
    let res = app.verify(&genesis, &block).await;
    assert!(matches!(res, Err(EvmAppError::StateRootMismatch { .. })));
}

## Lifecycle

### `test_genesis_to_verify`
- **Layer**: e2e
- **Crate**: `app`, `app-evm`
- **Input**: Fresh database
- **Assert**: Genesis -> Propose -> Verify sequence completes. **[PROPOSED]**: `app_evm::EvmApplication` lifecycle.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_genesis_to_verify() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    let (block, _res) = app.propose(&genesis, 1).await.unwrap();
    assert_eq!(block.parent_id, genesis.id());
    app.verify(&genesis, &block).await.unwrap();
}

### `test_error_propagation_through_adapter`
- **Layer**: e2e
- **Crate**: `app`, `app-evm`
- **Input**: Invalid block ID
- **Assert**: `verify` returns `ConsensusError::Verification`. **Grounded**: `crates/consensus/src/app.rs::ConsensusApp::verify`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_error_propagation_through_adapter() {
    let app = EvmApplication::new(db, spec);
    let adapter = ApplicationAdapter::new(app);
    let genesis = adapter.genesis().await;
    let block = adapter.propose(&genesis, 1).await.unwrap();
    // Tamper with block to force verify failure
    let mut bad_block = block.clone();
    bad_block.parent_id = [0xee; 32];
    let res = adapter.verify(&genesis, &bad_block).await;
    assert!(matches!(res, Err(ConsensusError::Verification(_))));
}
```
