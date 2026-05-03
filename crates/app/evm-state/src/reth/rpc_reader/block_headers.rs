use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::{
    RpcHeaderCarrierReader, RpcHeaderLookupReader, RpcHeaderRangeReader,
};

#[derive(Clone, Copy, Debug)]
pub struct RpcHeaderReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcHeaderReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn lookup(&self) -> RpcHeaderLookupReader<'a> {
        RpcHeaderLookupReader::new(self.db)
    }

    pub fn ranges(&self) -> RpcHeaderRangeReader<'a> {
        RpcHeaderRangeReader::new(self.db)
    }

    pub fn carriers(&self) -> RpcHeaderCarrierReader<'a> {
        RpcHeaderCarrierReader::new(self.db)
    }
}
