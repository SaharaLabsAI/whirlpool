//! CommonwarePeerId newtype that implements our PeerId trait.

use commonware_cryptography::PublicKey;
use p2p::PeerId;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A peer identifier wrapping a Commonware PublicKey.
///
/// This newtype adapter allows any Commonware-compatible `PublicKey` type
/// to be used as a `PeerId` in our vendor-agnostic networking layer.
///
/// # Example
/// ```ignore
/// use p2p_commonware::CommonwarePeerId;
/// use commonware_cryptography::ed25519::PublicKey;
///
/// let pk: PublicKey = // ... some public key
/// let peer_id = CommonwarePeerId(pk);
/// ```
#[derive(Clone, Debug)]
pub struct CommonwarePeerId<P: PublicKey>(pub P);

impl<P> PeerId for CommonwarePeerId<P> where
    P: PublicKey + Clone + Eq + Hash + fmt::Debug + Send + Sync + 'static
{
}

// Implement Eq trait (required by PeerId)
impl<P: PublicKey + Eq> Eq for CommonwarePeerId<P> {}

// Implement PartialEq trait
impl<P: PublicKey + PartialEq> PartialEq for CommonwarePeerId<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// Implement Hash trait (required by PeerId)
impl<P: PublicKey + Hash> Hash for CommonwarePeerId<P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the raw bytes representation of the public key
        self.0.as_ref().hash(state);
    }
}

impl<P: PublicKey> CommonwarePeerId<P> {
    /// Converts the peer ID to its byte representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_ref().to_vec()
    }
}
