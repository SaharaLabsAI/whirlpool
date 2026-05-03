use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::{
    RpcReceiptReader, RpcTransactionLookupReader, RpcTransactionMetaReader,
};

#[derive(Clone, Copy, Debug)]
pub struct RpcTransactionReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcTransactionReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn lookup(&self) -> RpcTransactionLookupReader<'a> {
        RpcTransactionLookupReader::new(self.db)
    }

    pub fn meta(&self) -> RpcTransactionMetaReader<'a> {
        RpcTransactionMetaReader::new(self.db)
    }

    pub fn receipts(&self) -> RpcReceiptReader<'a> {
        RpcReceiptReader::new(self.db)
    }
}
