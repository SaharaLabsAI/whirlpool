use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::{Block as VendorBlock, Heightable};
use commonware_cryptography::{Committable, Digestible};
use consensus::traits::Block as CoreBlock;

use super::{BlockDigest, BlockId, EvmBlock};

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
        buf.put_slice(&self.proposer_public_key);
        buf.put_slice(&self.proposer_fee_recipient);
        buf.put_u32(self.extra_data.len() as u32);
        buf.put_slice(&self.extra_data);
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
            + 32
            + 20
            + 4
            + self.extra_data.len()
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
        const MIN_ENCODED_BLOCK_LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 20 + 4 + 8 + 8 + 8 + 4;
        if reader.remaining() < MIN_ENCODED_BLOCK_LEN {
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

        let mut proposer_public_key = [0u8; 32];
        reader.copy_to_slice(&mut proposer_public_key);

        let mut proposer_fee_recipient = [0u8; 20];
        reader.copy_to_slice(&mut proposer_fee_recipient);

        if reader.remaining() < 4 {
            return Err(CodecError::Invalid("EvmBlock", "missing extra_data length"));
        }
        let extra_data_len = reader.get_u32() as usize;
        if reader.remaining() < extra_data_len {
            return Err(CodecError::Invalid(
                "EvmBlock",
                "extra_data exceeds remaining bytes",
            ));
        }
        let mut extra_data = vec![0u8; extra_data_len];
        reader.copy_to_slice(&mut extra_data);

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
            proposer_public_key,
            proposer_fee_recipient,
            extra_data,
            gas_used,
            base_fee_per_gas,
            timestamp,
            transactions,
        })
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
    fn parent(&self) -> Self::Digest {
        BlockDigest::from(self.parent_id)
    }
}
