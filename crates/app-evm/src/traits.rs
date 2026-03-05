use alloy_primitives::B256;
use revm::database::BundleState;
use state::traits::StateDb;

/// Trait for accessing state root from a database.
pub trait StateProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn state_root(&self) -> Result<B256, Self::Error>;
    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error>;
}

impl<T> StateProvider for T
where
    T: StateDb,
{
    type Error = <T as StateDb>::Error;

    fn state_root(&self) -> Result<B256, Self::Error> {
        StateDb::state_root(self)
    }

    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error> {
        StateDb::commit(self, bundle)
    }
}
