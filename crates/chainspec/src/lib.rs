pub mod community_pool;
pub mod genesis;
pub mod native_token;
pub mod validators;

pub const SAHARA_CHAIN_ID: u64 = 313_371;

#[cfg(test)]
mod native_token_tests;

#[cfg(test)]
mod tests;
