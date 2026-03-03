//! Configuration for the Commonware Simplex BFT consensus engine.

use std::num::NonZeroUsize;
use std::time::Duration;

use commonware_cryptography::ed25519::{PrivateKey as Ed25519Signer, PublicKey};

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

    /// Timeout for fetching blocks from peers.
    pub fetch_timeout: Duration,

    /// Maximum number of concurrent block fetch operations.
    pub fetch_concurrent: usize,

    /// Ed25519 signer used by this node to produce consensus signatures.
    pub signer: Ed25519Signer,

    /// Validator set public keys used for participation and verification.
    pub validators: Vec<PublicKey>,
}

#[cfg(test)]
mod tests {
    use super::CommonwareConfig;
    use commonware_cryptography::ed25519::PrivateKey;
    use commonware_cryptography::Signer as _;
    use std::num::NonZeroUsize;
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
            fetch_timeout: Duration::from_secs(1),
            fetch_concurrent: 4,
            signer,
            validators,
        };

        assert_eq!(config.validators.len(), 1);
    }
}
