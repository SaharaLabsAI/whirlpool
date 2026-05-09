use alloy_primitives::Address;
use std::collections::HashSet;

pub fn address_not_already_registered(already_registered: bool) -> bool {
    !already_registered
}

pub fn address_does_not_collide_with_builtin(collides_with_builtin: bool) -> bool {
    !collides_with_builtin
}

pub fn addresses_are_unique(addresses: &[Address]) -> bool {
    let mut seen = HashSet::with_capacity(addresses.len());
    addresses
        .iter()
        .copied()
        .all(|address| seen.insert(address))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;

    use crate::invariants::registry::{
        address_does_not_collide_with_builtin, address_not_already_registered, addresses_are_unique,
    };

    #[test]
    fn registry_address_predicates_are_fail_closed() {
        assert!(address_not_already_registered(false));
        assert!(!address_not_already_registered(true));
        assert!(address_does_not_collide_with_builtin(false));
        assert!(!address_does_not_collide_with_builtin(true));
    }

    #[test]
    fn detects_duplicate_addresses() {
        let one = Address::repeat_byte(1);
        let two = Address::repeat_byte(2);

        assert!(addresses_are_unique(&[one, two]));
        assert!(!addresses_are_unique(&[one, two, one]));
    }
}
