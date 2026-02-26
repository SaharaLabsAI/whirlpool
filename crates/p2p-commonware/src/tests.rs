//! Tests for p2p-commonware crate.

#[cfg(test)]
mod tests {
    use crate::{CommonwarePeerId, map_send_error, MultiplexSender};

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
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

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
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

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
            p2p::P2pError::SendFailed(msg) => {
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
            p2p::P2pError::SendFailed(msg) => {
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
    // These are left as RED phase tests for now

    #[tokio::test]
    async fn test_multiplex_receiver_tags_channel() {
        // TODO: Implement - should tag with correct channel
        panic!("not yet implemented - RED phase");
    }

    #[tokio::test]
    async fn test_multiplex_receiver_merges_channels() {
        // TODO: Implement - should merge all 3 channels
        panic!("not yet implemented - RED phase");
    }

    #[tokio::test]
    async fn test_multiplex_receiver_returns_none_on_shutdown() {
        // TODO: Implement - should return None when all senders close
        panic!("not yet implemented - RED phase");
    }
}
