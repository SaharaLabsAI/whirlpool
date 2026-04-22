use evm_precompiles::{decode_u64_storage_value, next_epoch_block_slot, EPOCH_PRECOMPILE_ADDRESS};

use crate::{error::EvmAppError, traits::StateDb};

pub use evm_precompiles::EpochBoundaryState;

pub fn load_epoch_boundary_state<DB>(db: &DB) -> Result<EpochBoundaryState, EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Into<EvmAppError>,
{
    let next_epoch_raw = db
        .get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
        .map_err(Into::into)?;
    let next_epoch_block = decode_u64_storage_value(next_epoch_raw).ok_or_else(|| {
        EvmAppError::InvalidBlock("epoch nextEpochBlock storage does not fit into u64".into())
    })?;

    Ok(EpochBoundaryState { next_epoch_block })
}
