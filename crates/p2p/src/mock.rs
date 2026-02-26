//! Mock implementations for testing P2P networking.
//!
//! This module provides simple in-memory mock implementations of the P2P
//! networking traits using tokio's mpsc channels for message passing.

use crate::{
    errors::P2pError,
    traits::{NetworkProvider, NetworkReceiver, NetworkSender, PeerId},
    types::{Channel, NetworkMessage, Recipients},
};
use bytes::Bytes;
use tokio::sync::mpsc;

/// A simple u64-based peer identifier for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MockPeerId(pub u64);

impl PeerId for MockPeerId {}

/// Mock network sender that delivers messages through an mpsc channel.
///
/// This sender can be cloned to create multiple handles that all send
/// to the same underlying receiver.
#[derive(Clone)]
pub struct MockSender {
    tx: mpsc::UnboundedSender<NetworkMessage<MockPeerId>>,
    peer_id: MockPeerId,
}

impl MockSender {
    /// Creates a new mock sender with the given channel and peer ID.
    pub fn new(tx: mpsc::UnboundedSender<NetworkMessage<MockPeerId>>, peer_id: MockPeerId) -> Self {
        Self { tx, peer_id }
    }
}

impl NetworkSender for MockSender {
    type PeerId = MockPeerId;

    async fn send(
        &self,
        channel: Channel,
        data: Bytes,
        _recipients: Recipients<Self::PeerId>,
    ) -> Result<(), P2pError> {
        let message = NetworkMessage::new(channel, data, self.peer_id);
        self.tx
            .send(message)
            .map_err(|_| P2pError::SendFailed("channel closed".to_string()))
    }
}

/// Mock network receiver that receives messages from an mpsc channel.
pub struct MockReceiver {
    rx: mpsc::UnboundedReceiver<NetworkMessage<MockPeerId>>,
}

impl MockReceiver {
    /// Creates a new mock receiver with the given channel.
    pub fn new(rx: mpsc::UnboundedReceiver<NetworkMessage<MockPeerId>>) -> Self {
        Self { rx }
    }
}

impl NetworkReceiver for MockReceiver {
    type PeerId = MockPeerId;

    async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>> {
        self.rx.recv().await
    }
}

/// Mock network provider that creates paired sender/receiver channels.
///
/// This provider creates in-memory channels using tokio's mpsc for testing
/// purposes. Each call to `start()` creates a new independent channel pair.
pub struct MockNetworkProvider {
    peer_id: MockPeerId,
}

impl MockNetworkProvider {
    /// Creates a new mock network provider with the specified peer ID.
    pub fn new(peer_id: MockPeerId) -> Self {
        Self { peer_id }
    }
}

impl NetworkProvider for MockNetworkProvider {
    type PeerId = MockPeerId;
    type Sender = MockSender;
    type Receiver = MockReceiver;

    fn start(self) -> Result<(Self::Sender, Self::Receiver), P2pError> {
        let (tx, rx) = mpsc::unbounded_channel();
        let sender = MockSender::new(tx, self.peer_id);
        let receiver = MockReceiver::new(rx);
        Ok((sender, receiver))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Channel;

    #[tokio::test]
    async fn test_mock_peer_id_implements_peer_id() {
        let peer = MockPeerId(42);
        let peer_clone = peer.clone();
        assert_eq!(peer, peer_clone);
    }

    #[tokio::test]
    async fn test_mock_sender_receiver_roundtrip() {
        let provider = MockNetworkProvider::new(MockPeerId(1));
        let (sender, mut receiver) = provider.start().unwrap();

        let test_data = Bytes::from("test message");
        let test_channel = Channel::VOTE;

        sender
            .send(test_channel, test_data.clone(), Recipients::All)
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.channel, test_channel);
        assert_eq!(received.data, test_data);
        assert_eq!(received.peer_id, MockPeerId(1));
    }

    #[tokio::test]
    async fn test_mock_sender_is_cloneable() {
        let provider = MockNetworkProvider::new(MockPeerId(2));
        let (sender, mut receiver) = provider.start().unwrap();

        let sender_clone = sender.clone();

        sender
            .send(Channel::CERTIFICATE, Bytes::from("msg1"), Recipients::All)
            .await
            .unwrap();

        sender_clone
            .send(Channel::RESOLVER, Bytes::from("msg2"), Recipients::All)
            .await
            .unwrap();

        let msg1 = receiver.recv().await.unwrap();
        assert_eq!(msg1.channel, Channel::CERTIFICATE);
        assert_eq!(msg1.data, Bytes::from("msg1"));

        let msg2 = receiver.recv().await.unwrap();
        assert_eq!(msg2.channel, Channel::RESOLVER);
        assert_eq!(msg2.data, Bytes::from("msg2"));
    }

    #[tokio::test]
    async fn test_mock_receiver_returns_none_when_sender_dropped() {
        let provider = MockNetworkProvider::new(MockPeerId(3));
        let (sender, mut receiver) = provider.start().unwrap();

        drop(sender);

        let result = receiver.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_multiple_messages() {
        let provider = MockNetworkProvider::new(MockPeerId(4));
        let (sender, mut receiver) = provider.start().unwrap();

        for i in 0..5 {
            sender
                .send(
                    Channel::VOTE,
                    Bytes::from(format!("message {}", i)),
                    Recipients::All,
                )
                .await
                .unwrap();
        }

        for i in 0..5 {
            let msg = receiver.recv().await.unwrap();
            assert_eq!(msg.data, Bytes::from(format!("message {}", i)));
        }
    }
}
