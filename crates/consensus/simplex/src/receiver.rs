// Inbound payload receiver — decodes PayloadRelayMessage frames from the PAYLOAD
// P2P channel, validates digest consistency, and persists accepted blocks into
// the shared BlockStore for later verification by the consensus engine.

use commonware_codec::Decode;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use network::NetworkReceiver;

use crate::mailbox::PayloadRelayMessage;
use crate::BlockStore;

/// Run the inbound payload receive loop.
///
/// Reads `NetworkMessage` frames from `receiver`, decodes each as a
/// [`PayloadRelayMessage`], validates the digest against the decoded block, and
/// stores valid blocks in `block_store`.  Malformed or mismatched messages are
/// logged and skipped — the loop never panics.
///
/// The loop runs until the receiver returns `None` (i.e. the sender half
/// is dropped, typically on engine shutdown).
pub async fn payload_receive_loop<B, R>(mut receiver: R, block_store: BlockStore<B>)
where
    B: Decode<Cfg = ()> + Digestible<Digest = Digest> + Clone + Send + Sync + 'static,
    R: NetworkReceiver + Send,
    R::PeerId: std::fmt::Debug,
{
    while let Some(msg) = receiver.recv().await {
        let peer_id = &msg.peer_id;
        let data = msg.data;

        // Decode wire envelope.
        let relay_msg = match PayloadRelayMessage::decode_wire(data) {
            Some(m) => m,
            None => {
                tracing::warn!(
                    ?peer_id,
                    "payload receiver: frame too short to decode, skipping"
                );
                continue;
            }
        };

        // Decode block bytes into B.
        let block = match B::decode_cfg(relay_msg.payload.clone(), &()) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    ?peer_id,
                    digest = ?relay_msg.digest,
                    error = %e,
                    "payload receiver: failed to decode block, skipping"
                );
                continue;
            }
        };

        // Validate digest: recompute from decoded block and compare.
        let computed_digest = block.digest();
        if computed_digest != relay_msg.digest {
            tracing::warn!(
                ?peer_id,
                expected = ?relay_msg.digest,
                actual = ?computed_digest,
                "payload receiver: digest mismatch, skipping"
            );
            continue;
        }

        // Store the validated block.
        let mut store = block_store.write().await;
        store.insert(computed_digest, block);

        tracing::debug!(
            ?peer_id,
            digest = ?computed_digest,
            "payload receiver: stored inbound block"
        );
    }

    tracing::info!("payload receive loop terminated (stream closed)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestBlock;
    use bytes::Bytes;
    use commonware_codec::Encode;
    use commonware_cryptography::sha256::Digest;
    use network::mock::MockPeerId;
    use network::types::{Channel, NetworkMessage};
    use network::NetworkReceiver;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn empty_block_store() -> BlockStore<TestBlock> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    /// A test receiver that yields pre-loaded messages then returns None.
    struct MockPayloadReceiver {
        messages: std::collections::VecDeque<NetworkMessage<MockPeerId>>,
    }

    impl MockPayloadReceiver {
        fn new(messages: Vec<NetworkMessage<MockPeerId>>) -> Self {
            Self {
                messages: messages.into(),
            }
        }
    }

    impl NetworkReceiver for MockPayloadReceiver {
        type PeerId = MockPeerId;

        async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>> {
            self.messages.pop_front()
        }
    }

    /// Helper: wrap raw bytes into a NetworkMessage with a dummy peer ID.
    fn make_network_message(data: Bytes) -> NetworkMessage<MockPeerId> {
        NetworkMessage {
            channel: Channel(3), // PAYLOAD
            data,
            peer_id: MockPeerId(42),
        }
    }

    #[tokio::test]
    async fn test_valid_inbound_payload_is_stored() {
        let block_store = empty_block_store();

        let block = TestBlock::genesis();
        let digest = block.digest();
        let relay_msg = PayloadRelayMessage::new(digest, block.encode());
        let wire = relay_msg.encode_wire();

        let receiver = MockPayloadReceiver::new(vec![make_network_message(wire)]);

        payload_receive_loop(receiver, Arc::clone(&block_store)).await;

        let store = block_store.read().await;
        assert!(store.contains_key(&digest), "block should be stored");
        assert_eq!(store.get(&digest).unwrap().digest(), digest);
    }

    #[tokio::test]
    async fn test_malformed_frame_is_skipped() {
        let block_store = empty_block_store();

        // Send a frame that's too short to be a valid PayloadRelayMessage
        let short_data = Bytes::from_static(&[0u8; 16]);
        let receiver = MockPayloadReceiver::new(vec![make_network_message(short_data)]);

        payload_receive_loop(receiver, Arc::clone(&block_store)).await;

        let store = block_store.read().await;
        assert!(store.is_empty(), "no block should be stored");
    }

    #[tokio::test]
    async fn test_digest_mismatch_is_skipped() {
        let block_store = empty_block_store();

        let block = TestBlock::genesis();
        // Use a WRONG digest in the envelope
        let wrong_digest = Digest::from([99u8; 32]);
        let relay_msg = PayloadRelayMessage::new(wrong_digest, block.encode());
        let wire = relay_msg.encode_wire();

        let receiver = MockPayloadReceiver::new(vec![make_network_message(wire)]);

        payload_receive_loop(receiver, Arc::clone(&block_store)).await;

        let store = block_store.read().await;
        assert!(store.is_empty(), "mismatched digest should not be stored");
    }

    #[tokio::test]
    async fn test_invalid_block_bytes_skipped() {
        let block_store = empty_block_store();

        // Valid-length digest + garbage block bytes
        let digest = Digest::from([1u8; 32]);
        let garbage_payload = Bytes::from_static(&[0xFF, 0xFE, 0xFD]);
        let relay_msg = PayloadRelayMessage::new(digest, garbage_payload);
        let wire = relay_msg.encode_wire();

        let receiver = MockPayloadReceiver::new(vec![make_network_message(wire)]);

        payload_receive_loop(receiver, Arc::clone(&block_store)).await;

        let store = block_store.read().await;
        assert!(store.is_empty(), "garbage block should not be stored");
    }

    #[tokio::test]
    async fn test_multiple_valid_payloads_stored() {
        let block_store = empty_block_store();

        let block1 = TestBlock::genesis();
        let digest1 = block1.digest();
        let msg1 = PayloadRelayMessage::new(digest1, block1.encode());

        let block2 = TestBlock::child_with_transactions(&block1, vec![vec![1, 2, 3]]);
        let digest2 = block2.digest();
        let msg2 = PayloadRelayMessage::new(digest2, block2.encode());

        let receiver = MockPayloadReceiver::new(vec![
            make_network_message(msg1.encode_wire()),
            make_network_message(msg2.encode_wire()),
        ]);

        payload_receive_loop(receiver, Arc::clone(&block_store)).await;

        let store = block_store.read().await;
        assert_eq!(store.len(), 2);
        assert!(store.contains_key(&digest1));
        assert!(store.contains_key(&digest2));
    }
}
