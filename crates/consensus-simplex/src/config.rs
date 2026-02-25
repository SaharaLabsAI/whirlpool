//! Configuration for the Commonware Simplex BFT consensus engine.

use std::num::NonZeroUsize;
use std::time::Duration;

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
}
