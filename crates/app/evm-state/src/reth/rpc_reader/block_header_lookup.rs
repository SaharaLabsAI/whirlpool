use alloy_consensus::Header;
use alloy_primitives::{BlockNumber, B256};
use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::{HeaderNumbers, Headers};

#[derive(Clone, Copy, Debug)]
pub struct RpcHeaderLookupReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcHeaderLookupReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn block_number(&self, hash: B256) -> Result<Option<BlockNumber>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<HeaderNumbers>(hash)
            .map_err(RethStateError::Database)
    }

    pub fn header_by_hash(&self, hash: B256) -> Result<Option<Header>, RethStateError> {
        let Some(number) = self.block_number(hash)? else {
            return Ok(None);
        };
        self.header_by_number(number)
    }

    pub fn header_by_number(&self, number: BlockNumber) -> Result<Option<Header>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<Headers>(number).map_err(RethStateError::Database)
    }
}
