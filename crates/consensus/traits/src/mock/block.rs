use crate::block::Block;

/// A mock block for testing. Uses `[u8; 32]` as the block identifier.
#[derive(Debug, Clone)]
pub struct MockBlock {
    pub id: [u8; 32],
    pub parent_id: [u8; 32],
    pub height: u64,
}

impl MockBlock {
    /// Create a genesis block (all zeros, height 0).
    pub fn genesis() -> Self {
        Self {
            id: [0u8; 32],
            parent_id: [0u8; 32],
            height: 0,
        }
    }

    /// Create a child block from a parent.
    /// The child's `id` is the parent's height + 1 encoded in the first 8 bytes.
    /// The child's `parent_id` is the parent's `id`.
    pub fn child(parent: &MockBlock) -> Self {
        let new_height = parent.height + 1;
        let mut id = [0u8; 32];
        id[..8].copy_from_slice(&new_height.to_le_bytes());
        Self {
            id,
            parent_id: parent.id,
            height: new_height,
        }
    }
}

impl Block for MockBlock {
    type Id = [u8; 32];

    fn id(&self) -> Self::Id {
        self.id
    }

    fn parent_id(&self) -> Self::Id {
        self.parent_id
    }

    fn height(&self) -> u64 {
        self.height
    }
}
