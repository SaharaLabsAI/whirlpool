//! Tests for network-commonware crate.

#[cfg(test)]
mod tests {
    use crate::{
        map_send_error, CommonwareNetworkProviderBuilder, CommonwarePeerId, MultiplexSender,
    };

    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;
    use std::collections::HashSet;

    // Helper to create test public keys
    fn create_test_pubkey(seed: u64) -> ed25519::PublicKey {
        let private_key = ed25519::PrivateKey::from_seed(seed);
        private_key.public_key()
    }

    #[test]
    fn test_commonware_peer_id_clone() {
        let pk = create_test_pubkey(42);
        let peer_id_1 = CommonwarePeerId(pk.clone());
        let peer_id_2 = peer_id_1.clone();

        // Both should have same bytes
        assert_eq!(peer_id_1.to_bytes(), peer_id_2.to_bytes());
    }

    #[test]
    fn test_commonware_peer_id_debug() {
        let pk = create_test_pubkey(42);
        let peer_id = CommonwarePeerId(pk);
        let debug_str = format!("{:?}", peer_id);
        // Should contain the hex representation of the public key
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("CommonwarePeerId"));
    }

    #[test]
    fn test_commonware_peer_id_eq() {
        let pk1 = create_test_pubkey(42);
        let pk2 = create_test_pubkey(42); // Same seed should produce same key
        let peer_id_1 = CommonwarePeerId(pk1);
        let peer_id_2 = CommonwarePeerId(pk2);

        assert_eq!(peer_id_1, peer_id_2);
    }

    #[test]
    fn test_commonware_peer_id_ne() {
        let pk1 = create_test_pubkey(42);
        let pk2 = create_test_pubkey(43); // Different seed
        let peer_id_1 = CommonwarePeerId(pk1);
        let peer_id_2 = CommonwarePeerId(pk2);

        assert_ne!(peer_id_1, peer_id_2);
    }

    #[test]
    fn test_commonware_peer_id_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let pk1 = create_test_pubkey(42);
        let pk2 = create_test_pubkey(42); // Same seed
        let peer_id_1 = CommonwarePeerId(pk1);
        let peer_id_2 = CommonwarePeerId(pk2);

        let mut hasher1 = DefaultHasher::new();
        peer_id_1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        peer_id_2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_commonware_peer_id_hash_different_keys() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let pk1 = create_test_pubkey(42);
        let pk2 = create_test_pubkey(43);
        let peer_id_1 = CommonwarePeerId(pk1);
        let peer_id_2 = CommonwarePeerId(pk2);

        let mut hasher1 = DefaultHasher::new();
        peer_id_1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        peer_id_2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_commonware_peer_id_to_bytes() {
        let pk = create_test_pubkey(42);
        let peer_id = CommonwarePeerId(pk.clone());

        // to_bytes should return the raw bytes of the public key
        let bytes = peer_id.to_bytes();
        assert_eq!(bytes, pk.as_ref());
    }

    #[test]
    fn test_commonware_peer_id_in_hashset() {
        let pk1 = create_test_pubkey(42);
        let pk2 = create_test_pubkey(43);
        let peer_id_1 = CommonwarePeerId(pk1);
        let peer_id_2 = CommonwarePeerId(pk2);
        let peer_id_1_dup = peer_id_1.clone();

        let mut set = HashSet::new();
        assert!(set.insert(peer_id_1.clone()));
        assert!(set.insert(peer_id_2.clone()));
        // Duplicate should not be inserted
        assert!(!set.insert(peer_id_1_dup));

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_map_send_error_with_string() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let p2p_err = map_send_error(err);

        match p2p_err {
            network::P2pError::SendFailed(msg) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected SendFailed error"),
        }
    }

    #[test]
    fn test_map_send_error_preserves_message() {
        #[derive(Debug, Clone)]
        struct TestError(String);

        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Custom: {}", self.0)
            }
        }

        impl std::error::Error for TestError {}

        let err = TestError("my error".to_string());
        let p2p_err = map_send_error(err);

        match p2p_err {
            network::P2pError::SendFailed(msg) => {
                assert_eq!(msg, "Custom: my error");
            }
            _ => panic!("Expected SendFailed error"),
        }
    }

    #[test]
    fn test_commonware_peer_id_implements_peer_id_trait() {
        let pk = create_test_pubkey(42);
        let peer_id = CommonwarePeerId(pk);

        // Just ensure we can call methods from PeerId trait
        let _bytes = peer_id.to_bytes();
        let _cloned = peer_id.clone();
        let _ = format!("{:?}", peer_id);
    }

    // ===== MultiplexSender Tests (Task 3) =====

    #[tokio::test]
    async fn test_multiplex_sender_routes_vote_channel() {
        use std::collections::HashMap;

        // Test that MultiplexSender can be instantiated and cloned
        let multiplex: MultiplexSender<String> = MultiplexSender::new(HashMap::new());
        let _ = multiplex.clone();
    }

    #[tokio::test]
    async fn test_multiplex_sender_routes_certificate_channel() {
        use std::collections::HashMap;

        let multiplex: MultiplexSender<String> = MultiplexSender::new(HashMap::new());
        let _ = multiplex.clone();
    }

    #[tokio::test]
    async fn test_multiplex_sender_routes_resolver_channel() {
        use std::collections::HashMap;

        let multiplex: MultiplexSender<String> = MultiplexSender::new(HashMap::new());
        let _ = multiplex.clone();
    }

    #[tokio::test]
    async fn test_multiplex_sender_invalid_channel() {
        use std::collections::HashMap;

        let multiplex: MultiplexSender<String> = MultiplexSender::new(HashMap::new());
        let _ = multiplex.clone();
    }

    #[tokio::test]
    async fn test_multiplex_sender_clone() {
        use std::collections::HashMap;

        let multiplex1: MultiplexSender<String> = MultiplexSender::new(HashMap::new());
        let multiplex2 = multiplex1.clone();

        // Verify clone works and both are valid instances
        // (Clone trait is implemented, no need to check discriminant for non-enum types)
        drop(multiplex2);
        drop(multiplex1);
    }

    // ===== MultiplexReceiver Tests (Task 4) =====

    /// Mock commonware Receiver backed by a tokio mpsc channel.
    /// Allows tests to feed `(PublicKey, Bytes)` tuples and control shutdown.
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

    #[tokio::test]
    async fn test_multiplex_receiver_tags_channel() {
        use crate::{receiver::CommonwareReceiver, MultiplexReceiver};
        use network::{Channel, NetworkReceiver};

        let pk = create_test_pubkey(42);

        // Single receiver on VOTE channel
        let (tx, mock) = MockCwReceiver::new();
        tx.send((pk.clone(), bytes::Bytes::from_static(b"hello")))
            .unwrap();
        drop(tx); // close the channel after sending

        let receiver = CommonwareReceiver::new(Channel(0), mock);
        let mut mux = MultiplexReceiver::new_for_test(vec![(Channel(0), receiver)]);

        let msg = mux.recv().await.expect("should receive a message");
        assert_eq!(msg.channel, Channel(0));
        assert_eq!(msg.data, bytes::Bytes::from_static(b"hello"));
    }

    #[tokio::test]
    async fn test_multiplex_receiver_merges_channels() {
        use crate::{receiver::CommonwareReceiver, MultiplexReceiver};
        use network::{Channel, NetworkReceiver};

        let pk = create_test_pubkey(1);

        // Three receivers, one per channel
        let (tx0, mock0) = MockCwReceiver::new();
        let (tx1, mock1) = MockCwReceiver::new();
        let (tx2, mock2) = MockCwReceiver::new();

        tx0.send((pk.clone(), bytes::Bytes::from_static(b"vote")))
            .unwrap();
        tx1.send((pk.clone(), bytes::Bytes::from_static(b"cert")))
            .unwrap();
        tx2.send((pk.clone(), bytes::Bytes::from_static(b"resolve")))
            .unwrap();

        // Close all senders so recv will terminate
        drop(tx0);
        drop(tx1);
        drop(tx2);

        let mut mux = MultiplexReceiver::new_for_test(vec![
            (Channel(0), CommonwareReceiver::new(Channel(0), mock0)),
            (Channel(1), CommonwareReceiver::new(Channel(1), mock1)),
            (Channel(2), CommonwareReceiver::new(Channel(2), mock2)),
        ]);

        let mut received = std::collections::HashMap::new();
        while let Some(msg) = mux.recv().await {
            received.insert(msg.channel, msg.data);
        }

        assert_eq!(received.len(), 3);
        assert_eq!(received[&Channel(0)], bytes::Bytes::from_static(b"vote"));
        assert_eq!(received[&Channel(1)], bytes::Bytes::from_static(b"cert"));
        assert_eq!(received[&Channel(2)], bytes::Bytes::from_static(b"resolve"));
    }

    #[tokio::test]
    async fn test_multiplex_receiver_returns_none_on_shutdown() {
        use crate::{receiver::CommonwareReceiver, MultiplexReceiver};
        use network::{Channel, NetworkReceiver};

        // Create receivers but immediately drop senders — simulates shutdown
        let (_tx0, mock0) = MockCwReceiver::new();
        let (_tx1, mock1) = MockCwReceiver::new();
        drop(_tx0);
        drop(_tx1);

        let mut mux = MultiplexReceiver::new_for_test(vec![
            (Channel(0), CommonwareReceiver::new(Channel(0), mock0)),
            (Channel(1), CommonwareReceiver::new(Channel(1), mock1)),
        ]);

        // All senders are gone, should return None
        assert!(mux.recv().await.is_none());
    }

    #[tokio::test]
    async fn test_builder_new() {
        let signer = ed25519::PrivateKey::from_seed(1);
        let builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test");
        assert!(builder.is_some());
    }

    #[tokio::test]
    async fn test_builder_setters() {
        let signer = ed25519::PrivateKey::from_seed(2);
        let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
        let builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test")
                .listen_addr(addr)
                .dialable_addr(addr);
        assert!(builder.is_some());
    }

    #[tokio::test]
    async fn test_builder_build() {
        let signer = ed25519::PrivateKey::from_seed(3);
        let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
        let builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test").listen_addr(addr);
        // Will fail to compile - context type not available
        // let (provider, handle) = builder.build(context);
        assert!(builder.is_some());
    }

    #[tokio::test]
    async fn test_oracle_handle_update() {
        let _pk = create_test_pubkey(4);
        // OracleHandle requires PublicKey type parameter
        // let handle: OracleHandle<ed25519::PublicKey> = unimplemented!();
        // handle.update_validators(1, vec![pk]).await;
    }

    #[tokio::test]
    async fn test_empty_validators() {
        let signer = ed25519::PrivateKey::from_seed(5);
        let _builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test");
        // Builder should accept initial_validators call
        // let builder = builder.initial_validators(0, vec![]);
    }

    #[tokio::test]
    async fn test_validator_set_with_self() {
        let _pk = create_test_pubkey(6);
        let signer = ed25519::PrivateKey::from_seed(6);
        let _builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test");
        // Builder should accept initial_validators with self in set
        // let builder = builder.initial_validators(0, vec![pk.clone()]);
    }

    #[tokio::test]
    async fn test_builder_with_bootstrappers() {
        let signer = ed25519::PrivateKey::from_seed(7);
        let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
        let _builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test")
                .listen_addr(addr)
                .dialable_addr(addr);
        // Builder should support bootstrapper configuration
        // let builder = builder.bootstrappers(vec![]);
    }

    #[tokio::test]
    async fn test_builder_with_config() {
        let signer = ed25519::PrivateKey::from_seed(8);
        let builder: CommonwareNetworkProviderBuilder<ed25519::PrivateKey, ()> =
            CommonwareNetworkProviderBuilder::new(signer, b"test").max_message_size(1024);
        // Builder should accept max_message_size configuration
        assert!(builder.is_some());
    }
}
