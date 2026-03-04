//! Transport-level abstractions for commonware-backed networking.

use std::fmt::Debug;
use std::hash::Hash;

use commonware_cryptography::PublicKey;
use commonware_p2p::authenticated::discovery::Oracle;
use p2p::P2pError;

use crate::provider::PerChannelNetwork;

/// Transport contract for providers that can expose dedicated simplex channels.
///
/// This trait is additive and mirrors the existing `CommonwareNetworkProvider`
/// transport surface so downstream users can depend on an interface boundary.
pub trait CommonwareTransport {
    type PublicKey: PublicKey + Clone + Hash + Eq + Debug + Send + Sync + 'static;
    type Sender;
    type Receiver;

    /// Start the transport and return dedicated vote/cert/resolver channel pairs.
    fn start_per_channel(self) -> Result<PerChannelNetwork<Self::Sender, Self::Receiver>, P2pError>;

    /// Access the discovery oracle used for validator/peer membership updates.
    fn oracle(&self) -> &Oracle<Self::PublicKey>;
}
