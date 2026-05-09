use alloy_primitives::{Address, U256};
use validators_reader::ValidatorEntry;

use crate::community_pool::*;

#[test]
fn build_effect_distributes_top_k_remainder() {
    let inputs = PostBlockAccountingInputs {
        boundary_required: true,
        gas_used: 0,
        base_fee_per_gas: 1,
        priority_fees: U256::ZERO,
        claim_recipient: Address::ZERO,
        simplex_validators: (1_u8..=5_u8)
            .map(|idx| ValidatorEntry {
                consensus_pubkey: [idx; 32],
                ethereum_address: Address::repeat_byte(idx),
            })
            .collect(),
    };
    let effect = build_post_block_accounting_effect(
        &inputs,
        1,
        CommunityPoolUnlockState {
            unlock_every_epochs: 1,
            unlock_amount_per_cycle: U256::from(10_u64),
            locked_remaining: U256::from(4_u64),
            last_processed_epoch: 0,
        },
    )
    .expect("build accounting effect");

    let unlock = effect
        .community_pool_unlock
        .expect("unlock effect should be present");
    assert_eq!(unlock.unlock_tranche, U256::from(4_u64));
    assert_eq!(unlock.next_locked_remaining, U256::ZERO);
    let expected = [1_u64, 1, 1, 1, 0];
    for (claim, expected) in unlock.validator_claims.iter().zip(expected) {
        assert_eq!(claim.amount, U256::from(expected));
    }
}

#[test]
fn unlock_invariant_rejects_unbounded_effect() {
    let state = CommunityPoolUnlockState {
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(5_u64),
        locked_remaining: U256::from(4_u64),
        last_processed_epoch: 0,
    };
    let effect = CommunityPoolUnlockEffect {
        unlock_tranche: U256::from(5_u64),
        validator_claims: vec![crate::fee_pool::ClaimCredit {
            recipient: Address::repeat_byte(1),
            amount: U256::from(5_u64),
        }],
        next_locked_remaining: U256::ZERO,
        last_processed_epoch: 1,
    };

    assert!(
        !crate::invariants::community_pool::unlock_effect_is_consistent(&state, 1, 1, &effect,)
    );
}
