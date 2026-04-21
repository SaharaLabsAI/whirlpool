use commonware_cryptography::Digestible;
use sha2::{Digest as Sha2Digest, Sha256};

use super::{BlockDigest, BlockId};

#[derive(Clone, Debug)]
pub struct EvmBlock {
    pub height: u64,
    pub parent_id: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub proposer_fee_recipient: [u8; 20],
    pub extra_data: Vec<u8>,
    pub gas_used: u64,
    pub base_fee_per_gas: u64,
    pub timestamp: u64,
    pub transactions: Vec<Vec<u8>>,
}

impl EvmBlock {
    pub fn compute_id(&self) -> BlockId {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.parent_id);
        hasher.update(self.state_root);
        hasher.update(self.transactions_root);
        hasher.update(self.proposer_public_key);
        hasher.update(self.proposer_fee_recipient);
        hasher.update((self.extra_data.len() as u32).to_le_bytes());
        hasher.update(&self.extra_data);

        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }

    fn compute_digest(&self) -> BlockDigest {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.parent_id);
        hasher.update(self.state_root);
        hasher.update(self.transactions_root);
        hasher.update(self.receipts_root);
        hasher.update(self.proposer_public_key);
        hasher.update(self.proposer_fee_recipient);
        hasher.update((self.extra_data.len() as u32).to_le_bytes());
        hasher.update(&self.extra_data);
        hasher.update(self.gas_used.to_le_bytes());
        hasher.update(self.base_fee_per_gas.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update((self.transactions.len() as u32).to_le_bytes());
        for tx in &self.transactions {
            hasher.update((tx.len() as u32).to_le_bytes());
            hasher.update(tx);
        }

        let digest = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest);
        BlockDigest::from(bytes)
    }
}

impl Digestible for EvmBlock {
    type Digest = BlockDigest;

    fn digest(&self) -> Self::Digest {
        self.compute_digest()
    }
}
