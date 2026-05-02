//! Compatibility facade for DKG transition context types.
//!
//! Canonical DKG transition-input ownership lives in `crate::context::dkg`.
//! Runtime validator membership is not config-owned; it is loaded by
//! `block_pipeline::validators` at proposal/verification timing.

mod activation_facade;
mod feature_facade;
mod payload_facade;

pub use crate::context::dkg::{
    CurrentFullDkgCandidate, DkgActivationOverrides, DkgTransitionConfig, FullDkgFeatureGate,
};
