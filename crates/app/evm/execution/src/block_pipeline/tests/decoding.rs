use super::*;
use crate::codec::{decode_evm_transaction, decode_evm_transactions};

#[test]
fn decode_evm_transaction_recovers_signer() {
    let (raw_tx, recovered) = sample_evm_tx();

    let decoded = decode_evm_transaction(&raw_tx).expect("tx should decode");

    assert_eq!(decoded.signer(), recovered);
}

#[test]
fn decode_evm_transactions_reject_invalid_bytes() {
    let err = decode_evm_transactions(&[vec![0xff, 0x00, 0x01]])
        .expect_err("invalid bytes should fail decoding");

    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[test]
fn decode_evm_transaction_rejects_trailing_bytes() {
    let (mut raw_tx, _recovered) = sample_evm_tx();
    raw_tx.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

    let err = decode_evm_transaction(&raw_tx).expect_err("trailing bytes should fail decoding");

    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn propose_executes_transfer_transaction() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, result) = app.propose(&parent, 1).await.unwrap();

    assert_eq!(block.transactions.len(), 1);
    assert!(result.gas_used > 0);
}

#[tokio::test]
async fn propose_rejects_padded_transaction_bytes_during_predecode() {
    let (mut raw_tx, _recovered) = sample_evm_tx();
    raw_tx.push(0x00);
    let (app, _db) = setup_app(vec![raw_tx]).await;
    let parent = app.genesis().await;

    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("padded tx bytes must fail proposal pre-decode");

    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn propose_invalid_transaction_records_false_and_excludes_tx() {
    let receiver = Address::with_last_byte(2);
    let (invalid_tx, recovered) =
        sample_evm_tx_with_chain_id(Some(SAHARA_CHAIN_ID + 1), 0, receiver);
    let (app, db) = setup_app(vec![invalid_tx.clone()]).await;
    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }
    let parent = app.genesis().await;

    let payload = app
        .propose_evm_transactions(&parent, &[invalid_tx], parent.timestamp + 12, 1)
        .expect("invalid tx should be soft-rejected during proposal");

    assert!(payload.included_user_transactions.is_empty());
    assert_eq!(payload.inclusion_outcomes, vec![false]);
}

#[tokio::test]
async fn verify_invalid_transaction_is_classified_as_invalid_block() {
    let receiver = Address::with_last_byte(2);
    let (invalid_tx, recovered) =
        sample_evm_tx_with_chain_id(Some(SAHARA_CHAIN_ID + 1), 0, receiver);
    let (app, db) = setup_app(vec![]).await;
    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }
    let parent = app.genesis().await;
    let (block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("empty block should propose");

    let err = app
        .verify_evm_transactions(&parent, &block, &[invalid_tx])
        .expect_err("invalid tx should be rejected as invalid block");

    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[test]
fn verify_non_validation_execution_failure_remains_execution() {
    let err = classify_tx_execution_error(BlockExecutionError::msg("execution unavailable"));

    match err {
        TxExecutionErrorDisposition::Other(message) => {
            let verify_error =
                EvmAppError::Execution(format!("Transaction execution failed: {message}"));
            assert!(matches!(verify_error, EvmAppError::Execution(_)));
        }
        TxExecutionErrorDisposition::InvalidTxValidation(_)
        | TxExecutionErrorDisposition::OtherValidation(_) => {
            panic!("internal execution failures must not be reclassified as invalid tx");
        }
    }
}

#[test]
fn verify_other_validation_execution_failure_is_classified_as_invalid_block() {
    let err = classify_tx_execution_error(BlockExecutionError::Validation(
        BlockValidationError::TransactionGasLimitMoreThanAvailableBlockGas {
            transaction_gas_limit: 30_000_001,
            block_available_gas: 30_000_000,
        },
    ));

    match err {
        TxExecutionErrorDisposition::OtherValidation(message) => {
            let verify_error = EvmAppError::InvalidBlock(format!(
                "Transaction execution failed validation: {message}"
            ));
            assert!(matches!(verify_error, EvmAppError::InvalidBlock(_)));
        }
        TxExecutionErrorDisposition::InvalidTxValidation(_)
        | TxExecutionErrorDisposition::Other(_) => {
            panic!("non-InvalidTx validation failures must still be classified as invalid blocks");
        }
    }
}
