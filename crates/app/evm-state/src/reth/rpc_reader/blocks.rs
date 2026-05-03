use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::{RpcBlockBodyReader, RpcCanonicalBlockReader, RpcHeaderReader};

#[derive(Clone, Copy, Debug)]
pub struct RpcBlockReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcBlockReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn canonical(&self) -> RpcCanonicalBlockReader<'a> {
        RpcCanonicalBlockReader::new(self.db)
    }

    pub fn headers(&self) -> RpcHeaderReader<'a> {
        RpcHeaderReader::new(self.db)
    }

    pub fn bodies(&self) -> RpcBlockBodyReader<'a> {
        RpcBlockBodyReader::new(self.db)
    }
}
