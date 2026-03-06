//! Commonware Receiver to NetworkReceiver adapter.

use crate::CommonwarePeerId;
use commonware_p2p::Receiver as CwReceiver;
use p2p::{Channel, NetworkMessage, NetworkReceiver};
use std::fmt::Debug;
use std::hash::Hash;

/// Adapts a Commonware Receiver to implement our vendor-agnostic NetworkReceiver trait.
///
/// This type wraps any Commonware `Receiver` implementation and bridges it to the `NetworkReceiver`
/// trait, allowing seamless integration with our consensus layer.
pub struct CommonwareReceiver<R> {
    inner: R,
}

impl<R> CommonwareReceiver<R> {
    /// Creates a new Commonware Receiver adapter.
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R> NetworkReceiver for CommonwareReceiver<R>
where
    R: CwReceiver + Send + 'static,
    R::PublicKey: Clone + Eq + Hash + Debug + Send + Sync + 'static,
{
    type PeerId = CommonwarePeerId<R::PublicKey>;

    async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>> {
        match self.inner.recv().await {
            Ok((peer_id, data)) => {
                Some(NetworkMessage {
                    channel: Channel(0), // TODO: extract channel from data if needed
                    data,
                    peer_id: CommonwarePeerId(peer_id),
                })
            }
            Err(_) => None,
        }
    }
}
