use std::collections::HashSet;

use alloy_primitives::Address;
use validators_reader::ValidatorEntry;

pub fn active_registry_entries_are_well_formed(entries: &[ValidatorEntry]) -> bool {
    let mut seen = HashSet::with_capacity(entries.len());
    entries.iter().all(|entry| {
        entry.consensus_pubkey != [0u8; 32]
            && seen.insert(entry.consensus_pubkey)
            && entry.ethereum_address != Address::ZERO
    })
}

pub fn registry_contains_proposer(
    entries: &[ValidatorEntry],
    proposer_public_key: &[u8; 32],
) -> bool {
    entries
        .iter()
        .any(|entry| &entry.consensus_pubkey == proposer_public_key)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use validators_reader::ValidatorEntry;

    use crate::invariants::validators::{
        active_registry_entries_are_well_formed, registry_contains_proposer,
    };

    #[test]
    fn validators_invariant_rejects_duplicate_pubkeys() {
        let entries = vec![
            ValidatorEntry {
                consensus_pubkey: [1; 32],
                ethereum_address: Address::repeat_byte(1),
            },
            ValidatorEntry {
                consensus_pubkey: [1; 32],
                ethereum_address: Address::repeat_byte(2),
            },
        ];

        assert!(!active_registry_entries_are_well_formed(&entries));
        assert!(registry_contains_proposer(&entries, &[1; 32]));
        assert!(!registry_contains_proposer(&entries, &[3; 32]));
    }
}
