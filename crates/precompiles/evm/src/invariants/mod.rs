//! Private semantic invariant predicates for Whirlpool system precompiles.
//!
//! These predicates are intentionally pure and value-oriented so they can become
//! direct formal-verification targets without coupling to REVM, `StateDb`, or
//! application-local runtime types.

pub mod accounting;
pub mod call_boundary;
pub mod community_pool;
pub mod epoch;
pub mod fee_pool;
pub mod registry;
pub mod validators;
