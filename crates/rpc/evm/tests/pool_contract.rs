//! TST-2: WhirlpoolTxPool satisfies RPC pool bounds and rejects blob transactions.

use std::sync::{Arc, Mutex};

use alloy_consensus::{TxEip1559, TxEip4844};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Signature, TxKind, U256};
use app_traits::traits::TxSource;
use reth_ethereum_primitives::{EthPrimitives, Transaction, TransactionSigned};
use reth_primitives_traits::SignedTransaction;
use reth_rpc_builder::RpcModuleBuilder;
use reth_transaction_pool::{
    error::PoolError, pool::AddedTransactionState, EthPooledTransaction, PoolTransaction,
    TransactionPool,
};
use rpc_eth::{pool::WhirlpoolTxPool, provider::WhirlpoolProvider};

#[derive(Debug, Default)]
struct RecordingTxSource {
    pushed: Mutex<Vec<Vec<u8>>>,
}

impl RecordingTxSource {
    fn pushed(&self) -> Vec<Vec<u8>> {
        self.pushed
            .lock()
            .expect("poisoned tx source mutex")
            .clone()
    }
}

impl TxSource for RecordingTxSource {
    fn push(&self, tx: Vec<u8>) {
        self.pushed
            .lock()
            .expect("poisoned tx source mutex")
            .push(tx);
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        self.pushed()
    }
}

fn _assert_pool_bounds() {
    fn assert_bounds<P>()
    where
        P: TransactionPool<Transaction = EthPooledTransaction> + Clone + Send + Sync + 'static,
    {
    }

    assert_bounds::<WhirlpoolTxPool>();
}

#[test]
fn pool_satisfies_rpc_node_core_bounds() {
    let _ = std::any::TypeId::of::<
        RpcModuleBuilder<EthPrimitives, WhirlpoolProvider, WhirlpoolTxPool, (), (), ()>,
    >();
    _assert_pool_bounds();
}

#[tokio::test]
async fn blob_transactions_are_rejected() {
    let tx_source = Arc::new(RecordingTxSource::default());
    let pool = WhirlpoolTxPool::new(tx_source.clone());
    let tx = pooled_tx(blob_signed_tx());
    let expected_hash = *tx.hash();

    let err = pool
        .add_external_transaction(tx)
        .await
        .expect_err("blob tx should be rejected");

    assert_eq!(err.hash, expected_hash);
    assert!(
        err.to_string()
            .contains("blob transactions (EIP-4844) are not supported"),
        "unexpected error: {err}"
    );
    assert!(
        tx_source.pushed().is_empty(),
        "blob tx must not be forwarded"
    );
}

#[tokio::test]
async fn non_blob_transactions_are_forwarded() {
    let tx_source = Arc::new(RecordingTxSource::default());
    let pool = WhirlpoolTxPool::new(tx_source.clone());
    let signed = eip1559_signed_tx();
    let expected = signed.clone().encoded_2718().to_vec();
    let tx = pooled_tx(signed);

    let outcome = pool
        .add_external_transaction(tx.clone())
        .await
        .expect("non-blob tx should be accepted");

    assert_eq!(outcome.hash, *tx.hash());
    assert_eq!(outcome.state, AddedTransactionState::Pending);
    assert_eq!(tx_source.pushed(), vec![expected]);
}

fn pooled_tx(signed: TransactionSigned) -> EthPooledTransaction {
    let recovered = signed
        .try_into_recovered()
        .expect("signed tx should recover");
    let encoded_length = recovered.encode_2718_len();
    EthPooledTransaction::new(recovered, encoded_length)
}

fn eip1559_signed_tx() -> TransactionSigned {
    TransactionSigned::new_unhashed(
        Transaction::Eip1559(TxEip1559 {
            chain_id: 1,
            nonce: 7,
            gas_limit: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            to: TxKind::Call(Address::repeat_byte(0x11)),
            value: U256::from(42_u64),
            access_list: Default::default(),
            input: Default::default(),
        }),
        Signature::test_signature(),
    )
}

fn blob_signed_tx() -> TransactionSigned {
    TransactionSigned::new_unhashed(
        Transaction::Eip4844(TxEip4844 {
            chain_id: 1,
            nonce: 9,
            gas_limit: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            to: Address::repeat_byte(0x22),
            value: U256::from(24_u64),
            access_list: Default::default(),
            input: Default::default(),
            blob_versioned_hashes: Default::default(),
            max_fee_per_blob_gas: 0,
        }),
        Signature::test_signature(),
    )
}

fn _pool_error_hash(err: &PoolError) -> alloy_primitives::TxHash {
    err.hash
}
