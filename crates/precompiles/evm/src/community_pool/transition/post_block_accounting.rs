use alloy_primitives::{Address, U256};
use validators_reader::ValidatorEntry;

use crate::fee_pool::ClaimCredit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunityPoolUnlockState {
    pub unlock_every_epochs: u64,
    pub unlock_amount_per_cycle: U256,
    pub locked_remaining: U256,
    pub last_processed_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunityPoolUnlockEffect {
    pub unlock_tranche: U256,
    pub validator_claims: Vec<ClaimCredit>,
    pub next_locked_remaining: U256,
    pub last_processed_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostBlockAccountingInputs {
    pub boundary_required: bool,
    pub gas_used: u64,
    pub base_fee_per_gas: u64,
    pub priority_fees: U256,
    pub claim_recipient: Address,
    pub simplex_validators: Vec<ValidatorEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostBlockAccountingEffect {
    pub burned_fees: U256,
    pub priority_fee_claim: Option<ClaimCredit>,
    pub community_pool_unlock: Option<CommunityPoolUnlockEffect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostBlockAccountingOutcome {
    pub current_epoch: u64,
    pub effect: PostBlockAccountingEffect,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostBlockAccountingEffectError {
    #[error("{0}")]
    InvalidStoredValue(String),
    #[error("{0}")]
    Execution(String),
}

pub fn build_post_block_accounting_effect(
    inputs: &PostBlockAccountingInputs,
    current_epoch: u64,
    unlock_state: CommunityPoolUnlockState,
) -> Result<PostBlockAccountingEffect, PostBlockAccountingEffectError> {
    let priority_fee_claim = (!inputs.priority_fees.is_zero()).then_some(ClaimCredit {
        recipient: inputs.claim_recipient,
        amount: inputs.priority_fees,
    });

    let effect = PostBlockAccountingEffect {
        burned_fees: U256::from(inputs.gas_used) * U256::from(inputs.base_fee_per_gas),
        priority_fee_claim,
        community_pool_unlock: build_community_pool_unlock_effect(
            inputs,
            current_epoch,
            unlock_state.clone(),
        )?,
    };
    if !crate::invariants::accounting::post_block_accounting_effect_matches_inputs(
        inputs,
        current_epoch,
        &unlock_state,
        &effect,
    ) {
        return Err(PostBlockAccountingEffectError::Execution(
            "post-block accounting invariant violation".into(),
        ));
    }

    Ok(effect)
}

fn build_community_pool_unlock_effect(
    inputs: &PostBlockAccountingInputs,
    current_epoch: u64,
    unlock_state: CommunityPoolUnlockState,
) -> Result<Option<CommunityPoolUnlockEffect>, PostBlockAccountingEffectError> {
    if !inputs.boundary_required {
        return Ok(None);
    }

    let unlock_enabled =
        unlock_state.unlock_every_epochs > 0 && !unlock_state.unlock_amount_per_cycle.is_zero();
    if !unlock_enabled {
        return Ok(None);
    }

    if inputs.simplex_validators.is_empty() {
        return Err(PostBlockAccountingEffectError::Execution(
            "community-pool unlock schedule enabled but simplex validators are empty".into(),
        ));
    }

    if current_epoch == 0 || !current_epoch.is_multiple_of(unlock_state.unlock_every_epochs) {
        return Ok(None);
    }

    if unlock_state.last_processed_epoch > current_epoch {
        return Err(PostBlockAccountingEffectError::InvalidStoredValue(format!(
            "community-pool lastProcessedEpoch {} exceeds current epoch {}",
            unlock_state.last_processed_epoch, current_epoch
        )));
    }
    if unlock_state.last_processed_epoch == current_epoch {
        return Ok(None);
    }

    if unlock_state.locked_remaining.is_zero() {
        let effect = CommunityPoolUnlockEffect {
            unlock_tranche: U256::ZERO,
            validator_claims: vec![],
            next_locked_remaining: unlock_state.locked_remaining,
            last_processed_epoch: current_epoch,
        };
        return Ok(Some(validate_unlock_effect(
            inputs,
            current_epoch,
            &unlock_state,
            effect,
        )?));
    }

    let unlock_tranche = unlock_state
        .unlock_amount_per_cycle
        .min(unlock_state.locked_remaining);
    if unlock_tranche.is_zero() {
        let effect = CommunityPoolUnlockEffect {
            unlock_tranche,
            validator_claims: vec![],
            next_locked_remaining: unlock_state.locked_remaining,
            last_processed_epoch: current_epoch,
        };
        return Ok(Some(validate_unlock_effect(
            inputs,
            current_epoch,
            &unlock_state,
            effect,
        )?));
    }

    let validator_claims = distribute_unlock_claims(unlock_tranche, &inputs.simplex_validators)?;
    let total_credited = validator_claims
        .iter()
        .fold(U256::ZERO, |acc, claim| acc + claim.amount);
    if total_credited != unlock_tranche {
        return Err(PostBlockAccountingEffectError::Execution(format!(
            "community-pool unlock accounting mismatch: credited {total_credited}, tranche {unlock_tranche}"
        )));
    }

    let next_locked_remaining = unlock_state
        .locked_remaining
        .checked_sub(unlock_tranche)
        .ok_or_else(|| {
            PostBlockAccountingEffectError::Execution("community-pool remaining underflow".into())
        })?;

    let effect = CommunityPoolUnlockEffect {
        unlock_tranche,
        validator_claims,
        next_locked_remaining,
        last_processed_epoch: current_epoch,
    };
    Ok(Some(validate_unlock_effect(
        inputs,
        current_epoch,
        &unlock_state,
        effect,
    )?))
}

fn validate_unlock_effect(
    inputs: &PostBlockAccountingInputs,
    current_epoch: u64,
    unlock_state: &CommunityPoolUnlockState,
    effect: CommunityPoolUnlockEffect,
) -> Result<CommunityPoolUnlockEffect, PostBlockAccountingEffectError> {
    if !crate::invariants::community_pool::unlock_effect_is_consistent(
        unlock_state,
        current_epoch,
        inputs.simplex_validators.len(),
        &effect,
    ) {
        return Err(PostBlockAccountingEffectError::Execution(
            "community-pool unlock invariant violation".into(),
        ));
    }

    Ok(effect)
}

fn distribute_unlock_claims(
    unlock_tranche: U256,
    simplex_validators: &[ValidatorEntry],
) -> Result<Vec<ClaimCredit>, PostBlockAccountingEffectError> {
    let validator_count = U256::from(u64::try_from(simplex_validators.len()).map_err(|_| {
        PostBlockAccountingEffectError::Execution("validator count does not fit into u64".into())
    })?);
    let base_share = unlock_tranche / validator_count;
    let remainder_u64 = u64::try_from(unlock_tranche % validator_count).map_err(|_| {
        PostBlockAccountingEffectError::Execution(
            "community-pool unlock remainder does not fit into u64".into(),
        )
    })?;
    let remainder = usize::try_from(remainder_u64).map_err(|_| {
        PostBlockAccountingEffectError::Execution(
            "community-pool unlock remainder does not fit into usize".into(),
        )
    })?;

    simplex_validators
        .iter()
        .enumerate()
        .map(|(index, validator)| {
            let extra = if index < remainder {
                U256::from(1_u64)
            } else {
                U256::ZERO
            };
            let amount = base_share.checked_add(extra).ok_or_else(|| {
                PostBlockAccountingEffectError::Execution("community-pool share overflow".into())
            })?;
            Ok(ClaimCredit {
                recipient: validator.ethereum_address,
                amount,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "post_block_accounting_tests.rs"]
mod tests;
