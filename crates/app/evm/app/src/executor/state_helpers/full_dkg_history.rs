use app::{decode_extra_data, ExtraDataDecodeMode, FullDkgV1};
use state::BlockStorage;

use crate::error::EvmAppError;

pub fn latest_committed_full_dkg<Storage>(
    storage: &Storage,
    start_height: u64,
) -> Result<Option<FullDkgV1>, EvmAppError>
where
    Storage: BlockStorage,
{
    let mut height = start_height;
    loop {
        let maybe_block = storage
            .get_block_by_number(height)
            .map_err(|err| EvmAppError::State(err.to_string()))?;
        if let Some(block) = maybe_block {
            let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Legacy)
                .map_err(|err| {
                    EvmAppError::InvalidBlock(format!(
                        "failed to decode historical block {height} extra_data: {err}"
                    ))
                })?;
            if let Some(full_dkg) = decoded.full_dkg {
                return Ok(Some(full_dkg));
            }
        }

        if height == 0 {
            break;
        }
        height -= 1;
    }

    Ok(None)
}
