mod error;

use std::sync::{Arc, Mutex, RwLock};

use alloy_primitives::bytes::BufMut;
use alloy_trie::root::ordered_trie_root_with_encoder;
use app::{
    traits::{Application, TxSource},
    EvmBlock, ExecutionResult, Receipt,
};
use app_evm::{EvmAppError, EvmApplication, WhirlpoolEvmConfig};
use state::BlockStorage;
use tx_dispatch::{classify_transactions, ClassifiedTransaction};

pub use app_evm::traits::StateProvider;
pub use error::CompositeAppError;

#[derive(Clone)]
pub struct CompositeApplication<DB> {
    evm_app: EvmApplication<DB>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
    pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>,
    last_proposed: Arc<Mutex<Option<(u64, EvmBlock, ExecutionResult, Vec<Receipt>)>>>,
}

impl<DB> CompositeApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_app: EvmApplication::new(evm_config, state_db, tx_source.clone()),
            tx_source,
            pending_receipts: Arc::new(Mutex::new(None)),
            last_proposed: Arc::new(Mutex::new(None)),
        }
    }

    pub fn store_finalized_block(
        &self,
        block: &EvmBlock,
        storage: &dyn BlockStorage,
    ) -> Result<(), CompositeAppError> {
        let receipts = {
            let mut guard = self.pending_receipts.lock().unwrap();
            guard.take().unwrap_or_default()
        };
        storage
            .store_block(block, &receipts)
            .map_err(|err| CompositeAppError::InvalidTransaction(err.to_string()))
    }
}

impl<DB> Application for CompositeApplication<DB>
where
    DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + std::fmt::Debug,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    type Block = EvmBlock;
    type Result = ExecutionResult;
    type Error = CompositeAppError;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        self.evm_app.genesis()
    }

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl std::future::Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send
    {
        async move {
            {
                let cache = self.last_proposed.lock().unwrap();
                if let Some((cached_height, ref block, ref result, ref receipts)) = *cache {
                    if cached_height == height {
                        let mut guard = self.pending_receipts.lock().unwrap();
                        *guard = Some(receipts.clone());
                        return Ok((block.clone(), result.clone()));
                    }
                }
            }

            let raw_pending = self.tx_source.pending();
            let classified_pending = classify_transactions(&raw_pending)
                .map_err(|err| CompositeAppError::InvalidTransaction(err.to_string()))?;

            let mut evm_candidates = Vec::new();
            for tx in &classified_pending {
                if let ClassifiedTransaction::Evm(raw_tx) = tx {
                    evm_candidates.push(raw_tx.clone());
                }
            }

            let timestamp = parent.timestamp + 12;
            let evm_payload =
                self.evm_app
                    .propose_evm_transactions(parent, &evm_candidates, timestamp)?;

            let mut executed_transactions = Vec::new();
            let mut inclusion_iter = evm_payload.inclusion_outcomes.iter();
            for tx in classified_pending {
                match tx {
                    ClassifiedTransaction::Mem(raw_tx) => executed_transactions.push(raw_tx),
                    ClassifiedTransaction::Evm(raw_tx) => {
                        if inclusion_iter.next().copied().unwrap_or(false) {
                            executed_transactions.push(raw_tx);
                        }
                    }
                }
            }

            let transactions_root =
                ordered_trie_root_with_encoder(&executed_transactions, |tx, out| {
                    out.put_slice(tx.as_slice());
                });

            {
                let mut guard = self.pending_receipts.lock().unwrap();
                *guard = Some(self.evm_app.pending_receipts());
            }

            let block = EvmBlock {
                height,
                parent_id: parent.compute_id(),
                state_root: evm_payload.result.state_root,
                transactions_root: transactions_root.0,
                receipts_root: evm_payload.result.receipts_root,
                proposer_public_key: evm_payload.proposer_public_key,
                proposer_fee_recipient: evm_payload.proposer_fee_recipient.into_array(),
                gas_used: evm_payload.result.gas_used,
                base_fee_per_gas: evm_payload.base_fee_per_gas,
                timestamp,
                transactions: executed_transactions,
            };

            {
                let mut cache = self.last_proposed.lock().unwrap();
                *cache = Some((
                    height,
                    block.clone(),
                    evm_payload.result.clone(),
                    evm_payload.receipts,
                ));
            }

            Ok((block, evm_payload.result))
        }
    }

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
        async move {
            let classified_txs = classify_transactions(&block.transactions)
                .map_err(|err| CompositeAppError::InvalidTransaction(err.to_string()))?;

            let computed_tx_root =
                ordered_trie_root_with_encoder(&block.transactions, |tx, out| out.put_slice(tx));
            if computed_tx_root.0 != block.transactions_root {
                return Err(CompositeAppError::InvalidTransaction(format!(
                    "Transactions root mismatch: expected {:?}, computed {:?}",
                    block.transactions_root, computed_tx_root.0
                )));
            }

            let mut evm_transactions = Vec::new();
            for tx in classified_txs {
                if let ClassifiedTransaction::Evm(raw_tx) = tx {
                    evm_transactions.push(raw_tx);
                }
            }

            let result = self
                .evm_app
                .verify_evm_transactions(parent, block, &evm_transactions)?;

            {
                let mut guard = self.pending_receipts.lock().unwrap();
                *guard = Some(self.evm_app.pending_receipts());
            }

            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
    use alloy_trie::EMPTY_ROOT_HASH;
    use app_mem::{PersonalityMarkdownTx, SignatureScheme};
    use chainspec::{build_sahara_chain_spec, SAHARA_CHAIN_ID};
    use reth_ethereum_primitives::TransactionSigned;
    use reth_primitives_traits::SignerRecoverable;
    use state_memory::InMemoryStateDb;

    struct MockTxSource {
        txs: Vec<Vec<u8>>,
    }

    impl TxSource for MockTxSource {
        fn push(&self, _tx: Vec<u8>) {}

        fn pending(&self) -> Vec<Vec<u8>> {
            self.txs.clone()
        }
    }

    async fn setup_app(
        txs: Vec<Vec<u8>>,
    ) -> (
        CompositeApplication<InMemoryStateDb>,
        Arc<RwLock<InMemoryStateDb>>,
    ) {
        let chain_spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let source = Arc::new(MockTxSource { txs });

        let app = CompositeApplication::new(config, db.clone(), source);
        (app, db)
    }

    fn sample_mem_tx() -> Vec<u8> {
        PersonalityMarkdownTx::new(
            b"signer-1".to_vec(),
            b"persona-1".to_vec(),
            7,
            b"# Persona\nBe precise.".to_vec(),
            SignatureScheme::RawSecp256k1,
            vec![0x11; 65],
        )
        .encode()
        .expect("mem tx should encode")
    }

    fn sample_evm_tx() -> (Vec<u8>, Address) {
        let receiver = Address::with_last_byte(2);
        let tx = TxLegacy {
            chain_id: Some(SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 2_000_000_000,
            gas_limit: 21_000,
            to: TxKind::Call(receiver),
            value: U256::from(1000),
            input: Bytes::default(),
        };
        let signature = Signature::test_signature();
        let signed: TransactionSigned = tx.into_signed(signature).into();
        let recovered = signed.recover_signer().unwrap();

        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);
        (encoded, recovered)
    }

    #[tokio::test]
    async fn propose_mixed_block_preserves_mem_and_executes_evm() {
        let mem_tx = sample_mem_tx();
        let (evm_tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![mem_tx.clone(), evm_tx.clone()]).await;

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

        assert_eq!(block.transactions, vec![mem_tx, evm_tx]);
        assert_eq!(result.receipt_count, 1);
    }

    #[tokio::test]
    async fn genesis_matches_evm_genesis() {
        let (app, _) = setup_app(vec![]).await;
        let genesis = app.genesis().await;

        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.transactions_root, EMPTY_ROOT_HASH.0);
    }
}
