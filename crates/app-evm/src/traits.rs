use alloy_primitives::{Address, B256};
use revm::database::BundleState;
use revm::state::AccountInfo;
use state::traits::StateDb;

/// Trait for accessing state root from a database.
pub trait StateProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn state_root(&self) -> Result<B256, Self::Error>;
    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error>;
    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;
    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error>;
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

    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        StateDb::get_account(self, address)
    }

    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error> {
        StateDb::insert_account(self, address, info)
    }
}
