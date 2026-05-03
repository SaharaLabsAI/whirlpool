use crate::reth::db::RethStateDb;
use app_primitives::header_extra_data::HeaderExtraDataHistory;

impl HeaderExtraDataHistory for RethStateDb {
    type Error = String;

    fn header_extra_data_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
        self.rpc_reader()
            .blocks()
            .headers()
            .carriers()
            .header_extra_data_at_height(height)
            .map_err(|err| err.to_string())
    }
}
