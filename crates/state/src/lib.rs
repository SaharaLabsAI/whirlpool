pub mod error;
pub mod traits;

// Re-export public types for convenience
pub use alloy_genesis::GenesisAccount;
pub use error::StateError;
pub use traits::StateDb;
