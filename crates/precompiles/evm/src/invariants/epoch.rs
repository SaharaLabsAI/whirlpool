use alloy_primitives::U256;

use crate::epoch::{
    current_epoch_slot, decode_epoch_start_block_storage_value, decode_u64_storage_value,
    epoch_start_block_slot, next_epoch_block_slot, EpochBoundaryEffect, EpochBoundaryStorageWrite,
};

pub fn advance_effect_is_consistent(
    current_epoch: u64,
    next_epoch_block: u64,
    epoch_blocks: u64,
    block_number: u64,
    writes: &[EpochBoundaryStorageWrite; 3],
) -> bool {
    if block_number != next_epoch_block {
        return false;
    }

    let Some(next_epoch) = current_epoch.checked_add(1) else {
        return false;
    };
    let Some(next_boundary) = next_epoch_block.checked_add(epoch_blocks) else {
        return false;
    };
    let Some(encoded_start) = block_number.checked_add(1) else {
        return false;
    };

    has_write(writes, current_epoch_slot(), U256::from(next_epoch))
        && has_write(writes, next_epoch_block_slot(), U256::from(next_boundary))
        && has_write(
            writes,
            epoch_start_block_slot(next_epoch),
            U256::from(encoded_start),
        )
}

pub fn boundary_effect_writes_known_storage_ready_values(effect: &EpochBoundaryEffect) -> bool {
    if !slots_are_unique(&effect.writes) {
        return false;
    }

    let Some(current_epoch_value) = find_write(&effect.writes, current_epoch_slot()) else {
        return false;
    };
    let Some(current_epoch) = decode_u64_storage_value(current_epoch_value) else {
        return false;
    };
    let Some(next_epoch_block_value) = find_write(&effect.writes, next_epoch_block_slot()) else {
        return false;
    };
    if decode_u64_storage_value(next_epoch_block_value).is_none() {
        return false;
    }

    let expected_epoch_start_slot = epoch_start_block_slot(current_epoch);
    let Some(epoch_start_value) = find_write(&effect.writes, expected_epoch_start_slot) else {
        return false;
    };

    decode_epoch_start_block_storage_value(epoch_start_value).is_some()
}

fn has_write(writes: &[EpochBoundaryStorageWrite; 3], slot: U256, value: U256) -> bool {
    writes
        .iter()
        .any(|write| write.slot == slot && write.value == value)
}

fn find_write(writes: &[EpochBoundaryStorageWrite; 3], slot: U256) -> Option<U256> {
    writes
        .iter()
        .find(|write| write.slot == slot)
        .map(|write| write.value)
}

fn slots_are_unique(writes: &[EpochBoundaryStorageWrite; 3]) -> bool {
    writes[0].slot != writes[1].slot
        && writes[0].slot != writes[2].slot
        && writes[1].slot != writes[2].slot
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use crate::epoch::{
        current_epoch_slot, epoch_start_block_slot, next_epoch_block_slot, EpochBoundaryEffect,
        EpochBoundaryStorageWrite,
    };
    use crate::invariants::epoch::{
        advance_effect_is_consistent, boundary_effect_writes_known_storage_ready_values,
    };

    #[test]
    fn detects_inconsistent_advance_effect() {
        let valid = [
            EpochBoundaryStorageWrite {
                slot: current_epoch_slot(),
                value: U256::from(1_u64),
            },
            EpochBoundaryStorageWrite {
                slot: next_epoch_block_slot(),
                value: U256::from(15_u64),
            },
            EpochBoundaryStorageWrite {
                slot: epoch_start_block_slot(1),
                value: U256::from(6_u64),
            },
        ];
        assert!(advance_effect_is_consistent(0, 5, 10, 5, &valid));

        let mut invalid = valid;
        invalid[1].value = U256::from(14_u64);
        assert!(!advance_effect_is_consistent(0, 5, 10, 5, &invalid));
    }

    #[test]
    fn detects_unknown_boundary_write_slot() {
        let effect = EpochBoundaryEffect {
            writes: [
                EpochBoundaryStorageWrite {
                    slot: current_epoch_slot(),
                    value: U256::from(1_u64),
                },
                EpochBoundaryStorageWrite {
                    slot: next_epoch_block_slot(),
                    value: U256::from(15_u64),
                },
                EpochBoundaryStorageWrite {
                    slot: U256::from(999_u64),
                    value: U256::from(6_u64),
                },
            ],
        };

        assert!(!boundary_effect_writes_known_storage_ready_values(&effect));
    }
}
