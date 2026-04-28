use state::BlockStorage;
use validators_dkg::{DkgExtraDataHistory, DkgMetadataError, FullDkgV1};

use crate::error::EvmAppError;

struct BlockStorageDkgHistory<'a, Storage> {
    storage: &'a Storage,
}

impl<Storage> DkgExtraDataHistory for BlockStorageDkgHistory<'_, Storage>
where
    Storage: BlockStorage,
{
    type Error = String;

    fn extra_data_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
        self.storage
            .get_block_by_number(height)
            .map(|maybe_block| maybe_block.map(|block| block.extra_data))
            .map_err(|err| err.to_string())
    }
}

pub fn latest_committed_full_dkg_from_storage<Storage>(
    storage: &Storage,
    start_height: u64,
) -> Result<Option<FullDkgV1>, EvmAppError>
where
    Storage: BlockStorage,
{
    validators_dkg::latest_committed_full_dkg(&BlockStorageDkgHistory { storage }, start_height)
        .map_err(|err| match err {
            DkgMetadataError::History(message) => EvmAppError::State(message),
            other => EvmAppError::InvalidBlock(other.to_string()),
        })
}
