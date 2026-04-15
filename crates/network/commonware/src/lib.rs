//! Commonware P2P bridge crate.
//!
//! This crate provides adapter types that bridge our vendor-agnostic `p2p` trait system
//! to the Commonware P2P implementation.
use bytes::Bytes;
use network::{Channel, NetworkMessage, NetworkReceiver, NetworkSender, P2pError, Recipients};
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
            .ok_or(P2pError::InvalidChannel(channel.0))?;
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

        // Poll all receivers in round-robin order once; return on first available message.
        for (_, receiver) in &mut self.receivers {
            if let Some(msg) = receiver.recv().await {
                // Trust receiver-owned channel tagging.
                return Some(msg);
            }
        }

        // All receivers returned None - they're all exhausted.
        None
    }
}

#[cfg(test)]
mod multiplex_receiver_contract_tests {
    use super::MultiplexReceiver;
    use crate::receiver::CommonwareReceiver;
    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;
    use network::{Channel, NetworkReceiver};

    #[derive(Debug)]
    struct MockCwReceiver {
        rx: tokio::sync::mpsc::UnboundedReceiver<(ed25519::PublicKey, bytes::Bytes)>,
    }

    impl MockCwReceiver {
        fn new() -> (
            tokio::sync::mpsc::UnboundedSender<(ed25519::PublicKey, bytes::Bytes)>,
            Self,
        ) {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, Self { rx })
        }
    }

    impl commonware_p2p::Receiver for MockCwReceiver {
        type Error = std::io::Error;
        type PublicKey = ed25519::PublicKey;

        async fn recv(&mut self) -> Result<(Self::PublicKey, bytes::Bytes), Self::Error> {
            self.rx.recv().await.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "channel closed")
            })
        }
    }

    fn create_test_pubkey(seed: u64) -> ed25519::PublicKey {
        let private_key = ed25519::PrivateKey::from_seed(seed);
        private_key.public_key()
    }

    #[tokio::test]
    async fn tst_req3_003_multiplex_forwards_receiver_tagged_channels() {
        let pk = create_test_pubkey(42);

        let (tx_vote, vote_mock) = MockCwReceiver::new();
        let (tx_cert, cert_mock) = MockCwReceiver::new();
        let (tx_resolver, resolver_mock) = MockCwReceiver::new();

        tx_vote
            .send((pk.clone(), bytes::Bytes::from_static(b"vote")))
            .expect("vote send succeeds");
        tx_cert
            .send((pk.clone(), bytes::Bytes::from_static(b"cert")))
            .expect("certificate send succeeds");
        tx_resolver
            .send((pk.clone(), bytes::Bytes::from_static(b"resolver")))
            .expect("resolver send succeeds");

        drop(tx_vote);
        drop(tx_cert);
        drop(tx_resolver);

        // Intentionally mismatch tuple channels to ensure mux forwards receiver tags unchanged.
        let mut mux = MultiplexReceiver::new_for_test(vec![
            (
                Channel::RESOLVER,
                CommonwareReceiver::new(Channel::VOTE, vote_mock),
            ),
            (
                Channel::VOTE,
                CommonwareReceiver::new(Channel::CERTIFICATE, cert_mock),
            ),
            (
                Channel::CERTIFICATE,
                CommonwareReceiver::new(Channel::RESOLVER, resolver_mock),
            ),
        ]);

        let msg1 = mux.recv().await.expect("vote message should be available");
        let msg2 = mux
            .recv()
            .await
            .expect("certificate message should be available");
        let msg3 = mux
            .recv()
            .await
            .expect("resolver message should be available");

        assert_eq!(msg1.channel, Channel::VOTE);
        assert_eq!(msg1.data, bytes::Bytes::from_static(b"vote"));

        assert_eq!(msg2.channel, Channel::CERTIFICATE);
        assert_eq!(msg2.data, bytes::Bytes::from_static(b"cert"));

        assert_eq!(msg3.channel, Channel::RESOLVER);
        assert_eq!(msg3.data, bytes::Bytes::from_static(b"resolver"));
    }
}
