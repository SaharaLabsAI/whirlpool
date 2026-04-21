use evm_precompiles::{next_epoch_block_slot, EPOCH_PRECOMPILE_ADDRESS};

use crate::{error::EvmAppError, traits::StateProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochBoundaryState {
    pub next_epoch_block: u64,
}

pub fn load_epoch_boundary_state<DB>(db: &DB) -> Result<EpochBoundaryState, EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let next_epoch_raw = db
        .get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
        .map_err(Into::into)?;
    let next_epoch_block = u64::try_from(next_epoch_raw).map_err(|_| {
        EvmAppError::InvalidBlock("epoch nextEpochBlock storage does not fit into u64".into())
    })?;

    Ok(EpochBoundaryState { next_epoch_block })
}
