use alloy_primitives::U256;

use crate::community_pool::{CommunityPoolUnlockEffect, CommunityPoolUnlockState};

pub fn unlock_effect_is_consistent(
    state: &CommunityPoolUnlockState,
    current_epoch: u64,
    validators_len: usize,
    effect: &CommunityPoolUnlockEffect,
) -> bool {
    if current_epoch == 0
        || state.unlock_every_epochs == 0
        || !current_epoch.is_multiple_of(state.unlock_every_epochs)
        || state.last_processed_epoch >= current_epoch
    {
        return false;
    }

    if effect.last_processed_epoch != current_epoch
        || effect.unlock_tranche > state.locked_remaining
        || state.locked_remaining.checked_sub(effect.unlock_tranche)
            != Some(effect.next_locked_remaining)
    {
        return false;
    }

    if !effect.unlock_tranche.is_zero() && validators_len == 0 {
        return false;
    }

    claim_amount_sum_matches_tranche(effect)
}

pub fn claim_amount_sum_matches_tranche(effect: &CommunityPoolUnlockEffect) -> bool {
    let Some(total) = effect
        .validator_claims
        .iter()
        .try_fold(U256::ZERO, |acc, claim| acc.checked_add(claim.amount))
    else {
        return false;
    };

    total == effect.unlock_tranche
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};

    use crate::community_pool::{CommunityPoolUnlockEffect, CommunityPoolUnlockState};
    use crate::fee_pool::ClaimCredit;
    use crate::invariants::community_pool::unlock_effect_is_consistent;

    #[test]
    fn unlock_invariant_rejects_unbounded_effect() {
        let state = CommunityPoolUnlockState {
            unlock_every_epochs: 2,
            unlock_amount_per_cycle: U256::from(5_u64),
            locked_remaining: U256::from(4_u64),
            last_processed_epoch: 0,
        };
        let effect = CommunityPoolUnlockEffect {
            unlock_tranche: U256::from(5_u64),
            validator_claims: vec![ClaimCredit {
                recipient: Address::repeat_byte(1),
                amount: U256::from(5_u64),
            }],
            next_locked_remaining: U256::ZERO,
            last_processed_epoch: 2,
        };

        assert!(!unlock_effect_is_consistent(&state, 2, 1, &effect));
    }
}
