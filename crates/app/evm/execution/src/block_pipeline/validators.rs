use std::fmt::Display;

use validators_dkg::ValidatorActivationSchedule;
use validators_reader::{ordered_consensus_pubkeys, ValidatorEntry};

use crate::block_pipeline::map_validators_runtime_error;
use crate::context::dkg::DkgTransitionConfig;
use crate::error::EvmAppError;
use crate::traits::StateDb;

#[derive(Debug)]
pub struct ActiveValidatorDkgInputs {
    pub entries: Vec<ValidatorEntry>,
    pub default_players: Vec<[u8; 32]>,
    pub activation_schedule: ValidatorActivationSchedule,
}

pub fn load_active_validator_dkg_inputs<DB>(
    db: &DB,
    dkg_transition: &DkgTransitionConfig,
) -> Result<ActiveValidatorDkgInputs, EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    let entries = evm_precompiles::load_active_validator_registry(db)
        .map_err(map_validators_runtime_error)?;
    active_validator_dkg_inputs_from_entries(entries, dkg_transition)
}

fn active_validator_dkg_inputs_from_entries(
    entries: Vec<ValidatorEntry>,
    dkg_transition: &DkgTransitionConfig,
) -> Result<ActiveValidatorDkgInputs, EvmAppError> {
    if entries.is_empty() {
        return Err(EvmAppError::InvalidBlock(
            "active validator registry is empty".into(),
        ));
    }

    let default_players = ordered_consensus_pubkeys(&entries);
    let activation_schedule =
        dkg_transition.activation_schedule_for_default_players(default_players.clone());

    Ok(ActiveValidatorDkgInputs {
        entries,
        default_players,
        activation_schedule,
    })
}
