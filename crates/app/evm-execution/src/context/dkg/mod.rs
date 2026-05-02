//! DKG transition-input context for block header metadata.
//!
//! This module holds node-local/current DKG inputs supplied to the block
//! pipeline, such as the locally available FullDKG candidate and explicit
//! activation override seeds. It does not read runtime validator state and does
//! not own DKG metadata semantics. Runtime validator reads stay in
//! `block_pipeline::validators`; metadata construction/validation stays in
//! `validators-dkg`.

mod activation_players;
mod feature_flags;
mod payload;

pub use activation_players::DkgActivationOverrides;
pub use feature_flags::FullDkgFeatureGate;
pub use payload::CurrentFullDkgCandidate;
use validators_dkg::{FullDkgOutputV1, ValidatorActivationSchedule};

#[derive(Debug, Clone, Default)]
pub struct DkgTransitionConfig {
    feature_gate: FullDkgFeatureGate,
    activation_overrides: DkgActivationOverrides,
    current_candidate: CurrentFullDkgCandidate,
}

// The crate-visible mutation helpers below are intentionally narrow: the
// public builder API remains on `WhirlpoolEvmConfig` without exposing extra
// public `DkgTransitionConfig` mutators.

impl DkgTransitionConfig {
    pub fn feature_gate(&self) -> &FullDkgFeatureGate {
        &self.feature_gate
    }

    pub fn current_candidate(&self) -> &CurrentFullDkgCandidate {
        &self.current_candidate
    }

    pub fn activation_schedule_for_default_players(
        &self,
        default_players: Vec<[u8; 32]>,
    ) -> ValidatorActivationSchedule {
        self.activation_overrides
            .schedule_for_default_players(default_players)
    }

    pub(crate) fn with_activation_players_for_epoch(
        mut self,
        epoch: u64,
        players: Vec<[u8; 32]>,
    ) -> Self {
        self.activation_overrides = self
            .activation_overrides
            .with_players_for_epoch(epoch, players);
        self
    }

    pub(crate) fn with_full_dkg_feature_enabled(mut self, enabled: bool) -> Self {
        self.feature_gate = self.feature_gate.with_enabled(enabled);
        self
    }

    pub(crate) fn with_current_full_dkg_output(mut self, output: FullDkgOutputV1) -> Self {
        self.current_candidate = self.current_candidate.with_output(output);
        self
    }
}
