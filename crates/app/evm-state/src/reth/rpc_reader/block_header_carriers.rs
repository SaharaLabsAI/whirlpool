use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::Headers;

#[derive(Clone, Copy, Debug)]
pub struct RpcHeaderCarrierReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcHeaderCarrierReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn header_extra_data_at_height(
        &self,
        height: u64,
    ) -> Result<Option<Vec<u8>>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let header = tx
            .get::<Headers>(height)
            .map_err(RethStateError::Database)?;
        Ok(header.map(|header| header.extra_data.to_vec()))
    }
}
