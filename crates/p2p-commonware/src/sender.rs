//! Commonware Sender to NetworkSender adapter.

use crate::{error::map_send_error, CommonwarePeerId};
use bytes::Bytes;
use commonware_p2p::{Recipients as CwRecipients, Sender as CwSender};
use p2p::{Channel, NetworkSender, P2pError, Recipients};
use std::fmt::Debug;
use std::hash::Hash;

/// Adapts a Commonware Sender to implement our vendor-agnostic NetworkSender trait.
///
/// This type wraps any Commonware `Sender` implementation and bridges it to the `NetworkSender`
/// trait, allowing seamless integration with our consensus layer.
#[derive(Clone)]
pub struct CommonwareSender<S> {
    inner: S,
}

impl<S> CommonwareSender<S> {
    /// Creates a new Commonware Sender adapter.
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> NetworkSender for CommonwareSender<S>
where
    S: CwSender + Clone + Send + Sync + 'static,
    S::PublicKey: Clone + Eq + Hash + Debug + Send + Sync + 'static,
{
    type PeerId = CommonwarePeerId<S::PublicKey>;

    async fn send(
        &self,
        _channel: Channel,
        data: Bytes,
        recipients: Recipients<Self::PeerId>,
    ) -> Result<(), P2pError> {
        // Convert Recipients from our vendor-agnostic type to Commonware's type
        let cw_recipients = match recipients {
            Recipients::All => CwRecipients::All,
            Recipients::One(peer) => CwRecipients::One(peer.0),
            Recipients::Many(peers) => CwRecipients::Some(peers.into_iter().map(|p| p.0).collect()),
        };

        // Clone self.inner to get a mutable reference (Commonware Sender trait requires &mut self)
        let mut sender = self.inner.clone();

        // Call commonware send (priority=false for now, as channels don't map directly)
        sender
            .send(cw_recipients, data, false)
            .await
            .map_err(map_send_error)?;

        Ok(())
    }
}
