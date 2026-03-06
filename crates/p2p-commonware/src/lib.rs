//! Commonware P2P bridge crate.
//!
//! This crate provides adapter types that bridge our vendor-agnostic `p2p` trait system
//! to the Commonware P2P implementation.
use bytes::Bytes;
use p2p::{Channel, NetworkMessage, NetworkReceiver, NetworkSender, P2pError, Recipients};
use std::collections::HashMap;
use std::sync::Arc;

pub mod provider;
pub mod traits;

mod error;
mod peer_id;

#[cfg(test)]
mod tests;

pub use error::{map_recv_error, map_send_error};
pub use peer_id::CommonwarePeerId;
pub use traits::CommonwareTransport;

pub mod receiver;
pub mod sender;
pub use commonware_p2p::authenticated::discovery::Bootstrapper;
pub use provider::{CommonwareNetworkProvider, CommonwareNetworkProviderBuilder, OracleHandle};
pub use receiver::CommonwareReceiver;
pub use sender::CommonwareSender;

// MultiplexSender: routes send() calls to correct per-channel CommonwareSender
#[derive(Clone)]
pub struct MultiplexSender<S> {
    senders: Arc<HashMap<Channel, CommonwareSender<S>>>,
}

impl<S> MultiplexSender<S> {
    pub fn new(senders: HashMap<Channel, CommonwareSender<S>>) -> Self {
        Self {
            senders: Arc::new(senders),
        }
    }
}

impl<S> NetworkSender for MultiplexSender<S>
where
    S: commonware_p2p::Sender + Clone + Send + Sync + 'static,
    S::PublicKey: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
{
    type PeerId = CommonwarePeerId<S::PublicKey>;

    async fn send(
        &self,
        channel: Channel,
        data: Bytes,
        recipients: Recipients<Self::PeerId>,
    ) -> Result<(), P2pError> {
        let sender = self
            .senders
            .get(&channel)
            .ok_or_else(|| P2pError::InvalidChannel(channel.0))?;
        sender.send(channel, data, recipients).await
    }
}

// MultiplexReceiver: merges multiple per-channel receivers into single stream
pub struct MultiplexReceiver<R> {
    receivers: Vec<(Channel, CommonwareReceiver<R>)>,
    _handle: Option<commonware_runtime::Handle<()>>,
}

impl<R> MultiplexReceiver<R> {
    pub fn new(
        receivers: Vec<(Channel, CommonwareReceiver<R>)>,
        handle: commonware_runtime::Handle<()>,
    ) -> Self {
        Self {
            receivers,
            _handle: Some(handle),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(receivers: Vec<(Channel, CommonwareReceiver<R>)>) -> Self {
        Self {
            receivers,
            _handle: None,
        }
    }
}

impl<R> NetworkReceiver for MultiplexReceiver<R>
where
    R: commonware_p2p::Receiver + Send + 'static,
    R::PublicKey: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
{
    type PeerId = CommonwarePeerId<R::PublicKey>;

    async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>> {
        if self.receivers.is_empty() {
            return None;
        }

        // Poll all receivers in round-robin fashion until we get a message or all are exhausted
        loop {
            if self.receivers.is_empty() {
                return None;
            }

            let len = self.receivers.len();

            // Try each receiver in order
            for i in 0..len {
                let (channel, receiver) = &mut self.receivers[i];
                match receiver.recv().await {
                    Some(msg) => {
                        // Fix the bug: tag with correct channel, not hardcoded Channel(0)
                        return Some(NetworkMessage {
                            channel: *channel,
                            data: msg.data,
                            peer_id: msg.peer_id,
                        });
                    }
                    None => {
                        // This receiver is done - continue checking others
                    }
                }
            }

            // All receivers returned None - they're all exhausted
            return None;
        }
    }
}
