use bytes::{BufMut, Bytes, BytesMut};
use commonware_cryptography::sha256::Digest;

/// Wire-format envelope for block payloads relayed over the PAYLOAD P2P channel.
///
/// Layout: `[32-byte SHA-256 digest][variable-length encoded block]`
///
/// The digest is placed first so the receiver can validate the block before
/// fully decoding it.  `encode_wire` / `decode_wire` handle serialisation
/// without pulling in an external framework.
pub struct PayloadRelayMessage {
    pub digest: Digest,
    pub payload: Bytes,
}

/// Fixed size of the digest prefix in the wire format (SHA-256 = 32 bytes).
pub const DIGEST_SIZE: usize = 32;

impl PayloadRelayMessage {
    /// Create a new relay message from a digest and pre-encoded block bytes.
    pub fn new(digest: Digest, payload: Bytes) -> Self {
        Self { digest, payload }
    }

    /// Serialise to wire format: `[digest bytes][payload bytes]`.
    pub fn encode_wire(&self) -> Bytes {
        let digest_bytes: &[u8] = self.digest.as_ref();
        let mut buf = BytesMut::with_capacity(DIGEST_SIZE + self.payload.len());
        buf.put_slice(digest_bytes);
        buf.put_slice(&self.payload);
        buf.freeze()
    }

    /// Deserialise from wire format.  Returns `None` if the buffer is too
    /// short to contain even the digest prefix.
    pub fn decode_wire(data: Bytes) -> Option<Self> {
        if data.len() < DIGEST_SIZE {
            return None;
        }
        let mut digest_arr = [0u8; DIGEST_SIZE];
        digest_arr.copy_from_slice(&data[..DIGEST_SIZE]);
        let digest = Digest::from(digest_arr);
        let payload = data.slice(DIGEST_SIZE..);
        Some(Self { digest, payload })
    }
}
