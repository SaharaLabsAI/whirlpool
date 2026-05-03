mod accounts;
mod block_bodies;
mod block_canonical;
mod block_header_carriers;
mod block_header_lookup;
mod block_header_ranges;
mod block_headers;
mod blocks;
mod dto;
mod transaction_lookup;
mod transaction_meta;
mod transaction_receipts;
mod transactions;

pub use accounts::RpcAccountReader;
pub use block_bodies::RpcBlockBodyReader;
pub use block_canonical::RpcCanonicalBlockReader;
pub use block_header_carriers::RpcHeaderCarrierReader;
pub use block_header_lookup::RpcHeaderLookupReader;
pub use block_header_ranges::RpcHeaderRangeReader;
pub use block_headers::RpcHeaderReader;
pub use blocks::RpcBlockReader;
pub use dto::{RpcBlockBodyIndices, RpcCanonicalTip, RpcStoredBlock, RpcTransactionMetaInputs};
pub use transaction_lookup::RpcTransactionLookupReader;
pub use transaction_meta::RpcTransactionMetaReader;
pub use transaction_receipts::RpcReceiptReader;
pub use transactions::RpcTransactionReader;

use crate::reth::db::RethStateDb;

#[derive(Clone, Copy, Debug)]
pub struct RpcStateReader<'a> {
    pub(in crate::reth) db: &'a RethStateDb,
}

impl<'a> RpcStateReader<'a> {
    pub fn blocks(&self) -> RpcBlockReader<'a> {
        RpcBlockReader::new(self.db)
    }

    pub fn transactions(&self) -> RpcTransactionReader<'a> {
        RpcTransactionReader::new(self.db)
    }

    pub fn accounts(&self) -> RpcAccountReader<'a> {
        RpcAccountReader::new(self.db)
    }
}
