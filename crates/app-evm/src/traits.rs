use alloy_primitives::B256;
use revm::database::BundleState;
use state::traits::StateDb;

/// Trait for accessing state root from a database.
pub trait StateProvider {
    fn state_root(&self) -> B256;
    fn commit(&mut self, bundle: &BundleState);
}

impl<T> StateProvider for T
where
    T: StateDb,
{
    fn state_root(&self) -> B256 {
        StateDb::state_root(self)
    }

    fn commit(&mut self, bundle: &BundleState) {
        StateDb::commit(self, bundle)
    }
}
