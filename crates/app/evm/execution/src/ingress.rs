use app_primitives::EvmBlock;
use app_traits::traits::TxSource;

pub fn pending_transactions(tx_source: &(dyn TxSource + Send + Sync)) -> Vec<Vec<u8>> {
    tx_source.pending()
}

pub fn candidate_block_transactions(block: &EvmBlock) -> &[Vec<u8>] {
    &block.transactions
}
