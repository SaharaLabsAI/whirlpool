use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::{Block as VendorBlock, Heightable};
use commonware_cryptography::{sha256, Committable, Digestible};
use consensus::traits::Block as CoreBlock;
use sha2::{Digest as Sha2Digest, Sha256};

pub type BlockId = [u8; 32];

type BlockDigest = sha256::Digest;

#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}

#[derive(Clone, Debug)]
pub struct EvmBlock {
    pub height: u64,
    pub parent_id: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
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

impl CoreBlock for EvmBlock {
    type Id = BlockId;

    fn id(&self) -> BlockId {
        self.compute_id()
    }

    fn parent_id(&self) -> BlockId {
        self.parent_id
    }

    fn height(&self) -> u64 {
        self.height
    }
}

impl CodecWrite for EvmBlock {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u64(self.height);
        buf.put_slice(&self.parent_id);
        buf.put_slice(&self.state_root);
        buf.put_slice(&self.transactions_root);
        buf.put_slice(&self.receipts_root);
        buf.put_u64(self.gas_used);
        buf.put_u64(self.base_fee_per_gas);
        buf.put_u64(self.timestamp);
        buf.put_u32(self.transactions.len() as u32);
        for tx in &self.transactions {
            buf.put_u32(tx.len() as u32);
            buf.put_slice(tx);
        }
    }
}

impl EncodeSize for EvmBlock {
    fn encode_size(&self) -> usize {
        8 + 32
            + 32
            + 32
            + 32
            + 8
            + 8
            + 8
            + 4
            + self
                .transactions
                .iter()
                .map(|tx| 4 + tx.len())
                .sum::<usize>()
    }
}

impl CodecRead for EvmBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, CodecError> {
        if reader.remaining() < 132 {
            return Err(CodecError::Invalid("EvmBlock", "not enough bytes"));
        }

        let height = reader.get_u64();

        let mut parent_id = [0u8; 32];
        reader.copy_to_slice(&mut parent_id);

        let mut state_root = [0u8; 32];
        reader.copy_to_slice(&mut state_root);

        let mut transactions_root = [0u8; 32];
        reader.copy_to_slice(&mut transactions_root);

        let mut receipts_root = [0u8; 32];
        reader.copy_to_slice(&mut receipts_root);

        let gas_used = reader.get_u64();
        let base_fee_per_gas = reader.get_u64();
        let timestamp = reader.get_u64();

        let tx_count = reader.get_u32() as usize;
        let mut transactions = Vec::with_capacity(tx_count);

        for _ in 0..tx_count {
            if reader.remaining() < 4 {
                return Err(CodecError::Invalid(
                    "EvmBlock",
                    "missing transaction length",
                ));
            }
            let tx_len = reader.get_u32() as usize;
            if reader.remaining() < tx_len {
                return Err(CodecError::Invalid(
                    "EvmBlock",
                    "transaction exceeds remaining bytes",
                ));
            }
            let mut tx = vec![0u8; tx_len];
            reader.copy_to_slice(&mut tx);
            transactions.push(tx);
        }

        Ok(Self {
            height,
            parent_id,
            state_root,
            transactions_root,
            receipts_root,
            gas_used,
            base_fee_per_gas,
            timestamp,
            transactions,
        })
    }
}

impl Digestible for EvmBlock {
    type Digest = BlockDigest;

    fn digest(&self) -> Self::Digest {
        self.compute_digest()
    }
}

impl Committable for EvmBlock {
    type Commitment = BlockDigest;

    fn commitment(&self) -> Self::Commitment {
        self.digest()
    }
}

impl Heightable for EvmBlock {
    fn height(&self) -> commonware_consensus::types::Height {
        commonware_consensus::types::Height::new(self.height)
    }
}

impl VendorBlock for EvmBlock {
    fn parent(&self) -> Self::Commitment {
        BlockDigest::from(self.parent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{EvmBlock, ExecutionResult};
    use consensus::traits::Block as CoreBlock;

    fn sample_block() -> EvmBlock {
        EvmBlock {
            height: 10,
            parent_id: [1u8; 32],
            state_root: [2u8; 32],
            transactions_root: [3u8; 32],
            receipts_root: [4u8; 32],
            gas_used: 42,
            base_fee_per_gas: 1_000_000_000,
            timestamp: 1_700_000_000,
            transactions: vec![vec![0xaa, 0xbb], vec![0xcc]],
        }
    }

    #[test]
    fn test_evm_block_trait_impl() {
        let block = sample_block();
        assert_eq!(CoreBlock::height(&block), 10);
        assert_eq!(CoreBlock::parent_id(&block), [1u8; 32]);
        assert!(CoreBlock::id(&block).iter().any(|b| *b != 0));
    }

    #[test]
    fn test_evm_block_codec_roundtrip() {
        use commonware_codec::{Read as CodecRead, Write as CodecWrite};

        let block = sample_block();
        let mut buf = bytes::BytesMut::new();
        block.write(&mut buf);
        let decoded = EvmBlock::read_cfg(&mut buf.freeze(), &()).expect("decode should succeed");

        assert_eq!(decoded.height, block.height);
        assert_eq!(decoded.parent_id, block.parent_id);
        assert_eq!(decoded.state_root, block.state_root);
        assert_eq!(decoded.transactions_root, block.transactions_root);
        assert_eq!(decoded.receipts_root, block.receipts_root);
        assert_eq!(decoded.gas_used, block.gas_used);
        assert_eq!(decoded.base_fee_per_gas, block.base_fee_per_gas);
        assert_eq!(decoded.timestamp, block.timestamp);
        assert_eq!(decoded.transactions, block.transactions);
    }

    #[test]
    fn test_execution_result_fields() {
        let result = ExecutionResult {
            state_root: [2u8; 32],
            receipts_root: [3u8; 32],
            gas_used: 100,
            receipt_count: 5,
        };

        assert_eq!(result.state_root, [2u8; 32]);
        assert_eq!(result.receipts_root, [3u8; 32]);
        assert_eq!(result.gas_used, 100);
        assert_eq!(result.receipt_count, 5);
    }
}
