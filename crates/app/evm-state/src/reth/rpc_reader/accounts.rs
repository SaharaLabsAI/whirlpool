use alloy_primitives::Address;
use reth_db::Database;
use reth_db_api::transaction::DbTx;
use reth_primitives_traits::Account;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::PlainAccountState;

#[derive(Clone, Copy, Debug)]
pub struct RpcAccountReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcAccountReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn basic_account(&self, address: Address) -> Result<Option<Account>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<PlainAccountState>(address)
            .map_err(RethStateError::Database)
    }
}
