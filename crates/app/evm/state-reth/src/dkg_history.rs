use reth_db::Database;
use reth_db_api::transaction::DbTx;
use validators_dkg::DkgHistory;

use crate::db::RethStateDb;
use crate::in_memory_db::InMemoryStateDb;
use crate::tables::Headers;

impl DkgHistory for RethStateDb {
    type Error = String;

    fn full_dkg_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
        let tx = self.inner().tx().map_err(|err| err.to_string())?;
        let header = tx.get::<Headers>(height).map_err(|err| err.to_string())?;
        Ok(header.map(|header| header.extra_data.to_vec()))
    }
}

impl DkgHistory for InMemoryStateDb {
    type Error = String;

    fn full_dkg_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
        self.raw_dkg_carrier_at_height(height)
    }
}

#[path = "tests/dkg_history.rs"]
#[cfg(test)]
mod tests;
