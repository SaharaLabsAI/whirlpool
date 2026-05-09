pub fn stateful_call_is_direct(is_direct_call: bool) -> bool {
    is_direct_call
}

pub fn write_call_is_not_static(is_static_call: bool) -> bool {
    !is_static_call
}

#[cfg(test)]
mod tests {
    use crate::invariants::call_boundary::{stateful_call_is_direct, write_call_is_not_static};

    #[test]
    fn call_boundary_predicates_are_fail_closed() {
        assert!(stateful_call_is_direct(true));
        assert!(!stateful_call_is_direct(false));
        assert!(write_call_is_not_static(false));
        assert!(!write_call_is_not_static(true));
    }
}
