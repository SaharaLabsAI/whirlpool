# App-EVM Crate Unit Tests
> Unit tests for app-evm crate interfaces.

Status: **PROPOSED** | Crate: `app-evm`

## Configuration

### `test_evm_config_chain_spec`
- **Layer**: unit
- **Crate**: `app-evm`
- **Input**: `WhirlpoolEvmConfig` with `ChainSpec`
- **Assert**: `chain_spec()` returns the provided spec. **Grounded**: `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm`.
- **Pseudo-code**:
```rust
#[test]
fn test_evm_config_chain_spec() {
    let spec = Arc::new(ChainSpec::default());
    let config = WhirlpoolEvmConfig::new(spec.clone());
    assert_eq!(config.chain_spec(), &spec);
}

## Application Lifecycle

### `test_evm_app_genesis`
- **Layer**: unit
- **Crate**: `app-evm`
- **Input**: `EvmApplication` instance
- **Assert**: `genesis()` block has height 0 and empty roots. **[PROPOSED]**: `app_evm::EvmApplication::genesis`.
- **Pseudo-code**:
```rust
#[tokio::test]
async fn test_evm_app_genesis() {
    let app = EvmApplication::new(db, spec);
    let genesis = app.genesis().await;
    assert_eq!(genesis.height, 0);
    assert_eq!(genesis.parent_id, [0; 32]);
}

## Error Handling

### `test_evm_app_error_mapping`
- **Layer**: unit
- **Crate**: `app-evm`
- **Input**: `EvmAppError::StateRootMismatch`
- **Assert**: Expected and computed roots are preserved via pattern matching. **[PROPOSED]**: `app_evm::EvmAppError` variants.
- **Pseudo-code**:
```rust
#[test]
fn test_evm_app_error_mapping() {
    let err = EvmAppError::StateRootMismatch {
        expected: [1; 32], computed: [2; 32],
    };
    match err {
        EvmAppError::StateRootMismatch { expected, computed } => {
            assert_eq!(expected, [1; 32]);
            assert_eq!(computed, [2; 32]);
        }
        _ => panic!("wrong variant"),
    }
}

### `test_evm_app_error_conversion`
- **Layer**: unit
- **Crate**: `app-evm`
- **Input**: `EvmAppError::State("fail".into())`
- **Assert**: Conversion to `ApplicationError` preserves message. **[PROPOSED]**: `app_evm::EvmAppError` into `app::ApplicationError`.
- **Pseudo-code**:
```rust
#[test]
fn test_evm_app_error_conversion() {
    let err = EvmAppError::State("fail".to_string());
    let app_err: ApplicationError = err.into();
    assert!(format!("{}", app_err).contains("fail"));
}
```

## Config Shape

### `test_evm_config_exposes_factory_and_assembler`
- **Layer**: unit
- **Crate**: `app-evm`
- **Input**: `WhirlpoolEvmConfig`
- **Assert**: Both `block_executor_factory()` and `block_assembler()` return valid references. **Grounded**: `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm`.
- **Pseudo-code**:
```rust
#[test]
fn test_evm_config_exposes_factory_and_assembler() {
    let config = WhirlpoolEvmConfig::new(spec);
    let _ = config.block_executor_factory();
    let _ = config.block_assembler();
    // Compile-time verification that both accessors exist and return expected types
}
```
