use std::collections::HashMap;

/// Canonical finalized personality entry stored by the prototype personality store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredPersonality {
    pub tx_hash: [u8; 32],
    pub block_height: u64,
    pub version: u8,
    pub signer: Vec<u8>,
    pub personality_id: Vec<u8>,
    pub nonce: u64,
    pub markdown: Vec<u8>,
    pub markdown_hash: [u8; 32],
    pub signature_scheme: u8,
    pub signature: Vec<u8>,
}

/// Storage for finalized personality data.
pub trait PersonalityStorage: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn put(&self, entry: StoredPersonality) -> Result<(), Self::Error>;
    fn get_latest(&self, personality_id: &[u8]) -> Result<Option<StoredPersonality>, Self::Error>;
    fn get_by_tx_hash(&self, tx_hash: &[u8]) -> Result<Option<StoredPersonality>, Self::Error>;
    fn get_by_signer_nonce(
        &self,
        signer: &[u8],
        nonce: u64,
    ) -> Result<Option<StoredPersonality>, Self::Error>;
    fn len(&self) -> Result<usize, Self::Error>;
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len()? == 0)
    }
}

pub type PersonalitySignerNonce = (Vec<u8>, u64);
pub type PersonalityLatestById = HashMap<Vec<u8>, StoredPersonality>;
pub type PersonalityByTxHash = HashMap<[u8; 32], StoredPersonality>;
pub type PersonalityBySignerNonce = HashMap<PersonalitySignerNonce, StoredPersonality>;
