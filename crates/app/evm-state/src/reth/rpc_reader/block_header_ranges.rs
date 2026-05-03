use alloy_consensus::Header;
use alloy_primitives::{BlockNumber, B256};
use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::{CanonicalHeaders, Headers};

#[derive(Clone, Copy, Debug)]
pub struct RpcHeaderRangeReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcHeaderRangeReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn headers_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> Result<Vec<Header>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut headers = Vec::new();
        for number in start..end {
            if let Some(header) = tx
                .get::<Headers>(number)
                .map_err(RethStateError::Database)?
            {
                headers.push(header);
            }
        }
        Ok(headers)
    }

    pub fn header_with_hash(
        &self,
        number: BlockNumber,
    ) -> Result<Option<(Header, B256)>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let Some(header) = tx
            .get::<Headers>(number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let Some(hash) = tx
            .get::<CanonicalHeaders>(number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        Ok(Some((header, hash)))
    }
}
