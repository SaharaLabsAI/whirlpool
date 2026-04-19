//! Configuration for the Commonware Simplex BFT consensus engine.

use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use commonware_cryptography::{
    bls12381::primitives::{group::Share, sharing::Sharing, variant::MinSig},
    ed25519::{PrivateKey as Ed25519Signer, PublicKey},
};

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SigningSchemeConfig {
    /// Use the legacy ed25519 simplex signing scheme.
    Ed25519 {
        signer: Ed25519Signer,
        validators: Vec<PublicKey>,
    },
    /// Use BLS threshold VRF signing while retaining ed25519 participant identities.
    BlsThresholdVrf {
        participants: Vec<PublicKey>,
        polynomial: Sharing<MinSig>,
        share: Share,
    },
}

impl SigningSchemeConfig {
    pub fn participants(&self) -> &[PublicKey] {
        match self {
            Self::Ed25519 { validators, .. } => validators,
            Self::BlsThresholdVrf { participants, .. } => participants,
        }
    }
}

/// Configuration for the Commonware Simplex BFT consensus engine.
///
/// This struct holds user-facing parameters needed to configure the consensus
/// engine. Internal construction details (scheme, elector, blocker, strategy,
/// buffer_pool, etc.) are handled by the engine implementation and not
/// exposed here.
pub struct CommonwareConfig {
    /// Namespace/partition prefix for storage isolation.
    pub namespace: String,

    /// Timeout for the leader to produce a proposal.
    pub leader_timeout: Duration,

    /// Timeout for collecting notarization signatures.
    pub notarization_timeout: Duration,

    /// Retry interval for nullification attempts.
    pub nullify_retry: Duration,

    /// Activity timeout in views before a validator is considered inactive.
    pub activity_timeout: u64,

    /// Skip timeout in views.
    pub skip_timeout: u64,

    /// Size of the internal mailbox buffer.
    pub mailbox_size: usize,

    /// Replay buffer size for message replay.
    pub replay_buffer: NonZeroUsize,

    /// Write buffer size for storage writes.
    pub write_buffer: NonZeroUsize,

    /// Starting epoch number.
    pub epoch: u64,

    /// Shared block-height tracker.
    ///
    /// The engine shares this `Arc<AtomicU64>` between its internal mailbox
    /// actor (which reads the current height to propose the next block) and
    /// the user-provided `EventSink` (which is responsible for advancing the
    /// value on finalization).  The caller should seed it from persistent
    /// storage so restarts resume at the correct height.
    pub height: Arc<AtomicU64>,

    /// Timeout for fetching blocks from peers.
    pub fetch_timeout: Duration,

    /// Maximum number of concurrent block fetch operations.
    pub fetch_concurrent: usize,

    /// Consensus signing scheme and key material.
    pub signing_scheme: SigningSchemeConfig,
}

#[cfg(test)]
mod tests {
    use super::{CommonwareConfig, SigningSchemeConfig};
    use commonware_cryptography::ed25519::PrivateKey;
    use commonware_cryptography::Signer as _;
    use std::num::NonZeroUsize;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_config_has_signer_and_validators() {
        let signer = PrivateKey::from_seed(7);
        let validators = vec![signer.public_key()];

        let config = CommonwareConfig {
            namespace: "test-consensus".to_string(),
            leader_timeout: Duration::from_secs(1),
            notarization_timeout: Duration::from_secs(1),
            nullify_retry: Duration::from_millis(100),
            activity_timeout: 10,
            skip_timeout: 5,
            mailbox_size: 16,
            replay_buffer: NonZeroUsize::new(16).unwrap(),
            write_buffer: NonZeroUsize::new(16).unwrap(),
            epoch: 0,
            height: Arc::new(AtomicU64::new(0)),
            fetch_timeout: Duration::from_secs(1),
            fetch_concurrent: 4,
            signing_scheme: SigningSchemeConfig::Ed25519 { signer, validators },
        };

        assert_eq!(config.signing_scheme.participants().len(), 1);
    }
}
