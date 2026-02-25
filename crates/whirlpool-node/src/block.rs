// EmptyBlock - Minimal block implementation for whirlpool-node

use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::Heightable;
use commonware_cryptography::{Committable, Digestible};
use consensus::Block as CoreBlock;
use sha2::{Digest as Sha2Digest, Sha256};

pub type BlockId = [u8; 32];

// Use vendor digest type
type BlockDigest = commonware_cryptography::sha256::Digest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmptyBlock {
    height: u64,
    parent_id: BlockId,
}

impl EmptyBlock {
    pub fn genesis() -> Self {
        Self {
            height: 0,
            parent_id: [0u8; 32],
        }
    }

    pub fn new(height: u64, parent_id: BlockId) -> Self {
        Self { height, parent_id }
    }

    fn compute_id(&self) -> BlockId {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(&self.parent_id);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }
}

// consensus::Block
impl CoreBlock for EmptyBlock {
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

// commonware_codec traits (follow TestBlock pattern exactly)
impl CodecWrite for EmptyBlock {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u64(self.height);
        buf.put_slice(&self.parent_id);
    }
}

impl EncodeSize for EmptyBlock {
    fn encode_size(&self) -> usize {
        8 + 32 // height + parent_id
    }
}

impl CodecRead for EmptyBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, CodecError> {
        if reader.remaining() < 40 {
            return Err(CodecError::Invalid("EmptyBlock", "not enough bytes"));
        }
        let height = reader.get_u64();
        let mut parent_id = [0u8; 32];
        reader.copy_to_slice(&mut parent_id);
        Ok(Self { height, parent_id })
    }
}

// commonware_cryptography traits
impl Digestible for EmptyBlock {
    type Digest = BlockDigest;

    fn digest(&self) -> Self::Digest {
        BlockDigest::from(self.compute_id())
    }
}

impl Committable for EmptyBlock {
    type Commitment = BlockDigest;

    fn commitment(&self) -> Self::Commitment {
        self.digest()
    }
}

// commonware_consensus traits
impl Heightable for EmptyBlock {
    fn height(&self) -> commonware_consensus::types::Height {
        commonware_consensus::types::Height::new(self.height)
    }
}

// EmptyBlock - Minimal block implementation for whirlpool-node

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_has_height_zero() {
        let genesis = EmptyBlock::genesis();
        assert_eq!(CoreBlock::height(&genesis), 0);
    }

    #[test]
    fn test_genesis_block_has_zero_parent() {
        let genesis = EmptyBlock::genesis();
        assert_eq!(genesis.parent_id(), [0u8; 32]);
    }

    #[test]
    fn test_genesis_block_id_is_deterministic() {
        let g1 = EmptyBlock::genesis();
        let g2 = EmptyBlock::genesis();
        assert_eq!(g1.id(), g2.id());
    }

    #[test]
    fn test_child_block_height_increments() {
        let parent = EmptyBlock::genesis();
        let child = EmptyBlock::new(1, parent.id());
        assert_eq!(CoreBlock::height(&child), 1);
    }

    #[test]
    fn test_child_block_references_parent() {
        let parent = EmptyBlock::genesis();
        let parent_id = parent.id();
        let child = EmptyBlock::new(1, parent_id);
        assert_eq!(child.parent_id(), parent_id);
    }

    #[test]
    fn test_codec_roundtrip() {
        use bytes::BytesMut;
        use commonware_codec::{Read as CodecRead, Write as CodecWrite};

        let block = EmptyBlock::new(5, [42u8; 32]);
        let mut buf = BytesMut::new();
        block.write(&mut buf);
        let decoded = EmptyBlock::read_cfg(&mut buf.freeze(), &()).unwrap();
        assert_eq!(CoreBlock::height(&decoded), CoreBlock::height(&block));
        assert_eq!(decoded.parent_id(), block.parent_id());
    }

    #[test]
    fn test_digest_deterministic() {
        use commonware_cryptography::Digestible;

        let block = EmptyBlock::new(3, [7u8; 32]);
        let d1 = block.digest();
        let d2 = block.digest();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_different_blocks_different_digests() {
        use commonware_cryptography::Digestible;

        let b1 = EmptyBlock::new(1, [0u8; 32]);
        let b2 = EmptyBlock::new(2, [0u8; 32]);
        assert_ne!(b1.digest(), b2.digest());
    }
}
