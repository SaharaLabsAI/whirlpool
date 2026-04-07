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
    channel: Channel,
    inner: R,
}

impl<R> CommonwareReceiver<R> {
    /// Creates a new Commonware Receiver adapter.
    pub fn new(channel: Channel, inner: R) -> Self {
        Self { channel, inner }
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
            Ok((peer_id, data)) => Some(NetworkMessage {
                channel: self.channel,
                data,
                peer_id: CommonwarePeerId(peer_id),
            }),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommonwareReceiver;
    use bytes::Bytes;
    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;
    use p2p::{Channel, NetworkReceiver};

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
    async fn tst_req3_001_receiver_emits_vote_channel() {
        let pk = create_test_pubkey(42);
        let payload = Bytes::from_static(b"vote-message");
        let (tx, mock) = MockCwReceiver::new();
        tx.send((pk.clone(), payload.clone()))
            .expect("send succeeds");
        drop(tx);

        let mut receiver = CommonwareReceiver::new(Channel::VOTE, mock);
        let message = receiver.recv().await.expect("message should be available");

        assert_eq!(message.channel, Channel::VOTE);
        assert_eq!(message.data, payload);
        assert_eq!(message.peer_id.0, pk);
    }

    #[tokio::test]
    async fn tst_req3_002_receiver_emits_distinct_channels() {
        let pk_a = create_test_pubkey(100);
        let pk_b = create_test_pubkey(200);

        let cert_payload = Bytes::from_static(b"certificate");
        let resolver_payload = Bytes::from_static(b"resolver");

        let (tx_cert, cert_mock) = MockCwReceiver::new();
        tx_cert
            .send((pk_a.clone(), cert_payload.clone()))
            .expect("cert send succeeds");
        drop(tx_cert);

        let (tx_resolver, resolver_mock) = MockCwReceiver::new();
        tx_resolver
            .send((pk_b.clone(), resolver_payload.clone()))
            .expect("resolver send succeeds");
        drop(tx_resolver);

        let mut cert_receiver = CommonwareReceiver::new(Channel::CERTIFICATE, cert_mock);
        let mut resolver_receiver = CommonwareReceiver::new(Channel::RESOLVER, resolver_mock);

        let cert_message = cert_receiver
            .recv()
            .await
            .expect("certificate message should be available");
        let resolver_message = resolver_receiver
            .recv()
            .await
            .expect("resolver message should be available");

        assert_eq!(cert_message.channel, Channel::CERTIFICATE);
        assert_eq!(resolver_message.channel, Channel::RESOLVER);
        assert_ne!(cert_message.channel, resolver_message.channel);

        assert_eq!(cert_message.data, cert_payload);
        assert_eq!(resolver_message.data, resolver_payload);

        assert_eq!(cert_message.peer_id.0, pk_a);
        assert_eq!(resolver_message.peer_id.0, pk_b);
    }
}
