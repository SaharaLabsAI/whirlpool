//! EmptyBlockApp - Stateless consensus application for empty blocks
//!
//! This module implements a minimal `ConsensusApp` using `EmptyBlock` as the block type.
//! The application is stateless (zero-sized struct) and enforces 5 block verification rules.

use crate::block::EmptyBlock;
use consensus::{Block as CoreBlock, ConsensusApp, ConsensusError};

/// Stateless consensus application for EmptyBlock.
///
/// This application implements the `ConsensusApp` trait with minimal logic:
/// - Genesis block at height 0 with zero parent
/// - Block proposals increment height sequentially
/// - Block verification enforces 5 rules (height, parent_id, self-reference, genesis constraints)
#[derive(Clone)]
pub struct EmptyBlockApp;

impl EmptyBlockApp {
    /// Create a new EmptyBlockApp instance.
    pub fn new() -> Self {
        Self
    }
}

impl ConsensusApp for EmptyBlockApp {
    type Block = EmptyBlock;

    async fn genesis(&self) -> EmptyBlock {
        EmptyBlock::genesis()
    }

    async fn propose(&self, parent: &EmptyBlock, height: u64) -> Option<EmptyBlock> {
        Some(EmptyBlock::new(height, CoreBlock::id(parent)))
    }

    async fn verify(&self, parent: &EmptyBlock, block: &EmptyBlock) -> Result<(), ConsensusError> {
        // Rule 1: Height must be parent + 1
        if CoreBlock::height(block) != CoreBlock::height(parent) + 1 {
            return Err(ConsensusError::InvalidBlock(format!(
                "height mismatch: expected {}, got {}",
                CoreBlock::height(parent) + 1,
                CoreBlock::height(block)
            )));
        }

        // Rule 2: parent_id must match
        if CoreBlock::parent_id(block) != CoreBlock::id(parent) {
            return Err(ConsensusError::InvalidBlock("parent mismatch".to_string()));
        }

        // Rule 3: No self-reference (except genesis)
        if CoreBlock::id(block) == CoreBlock::parent_id(block) && CoreBlock::height(block) != 0 {
            return Err(ConsensusError::InvalidBlock(
                "non-genesis block self-references".to_string(),
            ));
        }

        // Rule 4: Height 0 with non-zero parent is invalid
        if CoreBlock::height(block) == 0 && CoreBlock::parent_id(block) != [0u8; 32] {
            return Err(ConsensusError::InvalidBlock(
                "height 0 with non-zero parent".to_string(),
            ));
        }

        // Rule 5: Implicit — genesis has zero parent (covered by rules above)

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Genesis returns EmptyBlock at height 0
    #[tokio::test]
    async fn test_genesis_returns_empty_block_at_height_zero() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        assert_eq!(CoreBlock::height(&genesis), 0);
    }

    // Test 2: Propose returns block at correct height
    #[tokio::test]
    async fn test_propose_returns_block_at_correct_height() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        let block = app
            .propose(&genesis, 1)
            .await
            .expect("propose should return Some");
        assert_eq!(CoreBlock::height(&block), 1);
    }

    // Test 3: Propose references parent correctly
    #[tokio::test]
    async fn test_propose_references_parent() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        let parent_id = CoreBlock::id(&genesis);
        let block = app
            .propose(&genesis, 1)
            .await
            .expect("propose should return Some");
        assert_eq!(CoreBlock::parent_id(&block), parent_id);
    }

    // Test 4: Verify valid block succeeds
    #[tokio::test]
    async fn test_verify_valid_block_succeeds() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        let block = EmptyBlock::new(1, CoreBlock::id(&genesis));
        assert!(app.verify(&genesis, &block).await.is_ok());
    }

    // Test 5: Verify wrong height fails
    #[tokio::test]
    async fn test_verify_wrong_height_fails() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        // Block claims height 5 instead of 1
        let block = EmptyBlock::new(5, CoreBlock::id(&genesis));
        let result = app.verify(&genesis, &block).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConsensusError::InvalidBlock(_)
        ));
    }

    // Test 6: Verify wrong parent fails
    #[tokio::test]
    async fn test_verify_wrong_parent_fails() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        // Block references wrong parent
        let block = EmptyBlock::new(1, [99u8; 32]);
        let result = app.verify(&genesis, &block).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConsensusError::InvalidBlock(_)
        ));
    }

    // Test 7: Verify genesis height with non-zero parent fails
    #[tokio::test]
    async fn test_verify_genesis_height_nonzero_fails() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        // Block at height 0 but with non-zero parent (invalid)
        let bad_genesis = EmptyBlock::new(0, [1u8; 32]);
        let result = app.verify(&genesis, &bad_genesis).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConsensusError::InvalidBlock(_)
        ));
    }

    // Test 8: Verify self-referencing block fails (non-genesis)
    #[tokio::test]
    async fn test_verify_self_referencing_fails() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        // Create a block that references itself
        let self_ref_block = EmptyBlock::new(1, [0u8; 32]);
        let self_ref_id = CoreBlock::id(&self_ref_block);
        let self_ref_block = EmptyBlock::new(1, self_ref_id);
        let result = app.verify(&genesis, &self_ref_block).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConsensusError::InvalidBlock(_)
        ));
    }

    // Test 9: Verify future height fails (height > parent + 1)
    #[tokio::test]
    async fn test_verify_future_height_fails() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        // Block jumps from height 0 to height 3 (invalid)
        let block = EmptyBlock::new(3, CoreBlock::id(&genesis));
        let result = app.verify(&genesis, &block).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConsensusError::InvalidBlock(_)
        ));
    }

    // Test 10: Propose after propose increments height correctly
    #[tokio::test]
    async fn test_propose_after_propose_increments() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        let block1 = app.propose(&genesis, 1).await.expect("first propose");
        let block2 = app.propose(&block1, 2).await.expect("second propose");
        assert_eq!(CoreBlock::height(&block1), 1);
        assert_eq!(CoreBlock::height(&block2), 2);
        assert_eq!(CoreBlock::parent_id(&block2), CoreBlock::id(&block1));
    }

    // Test 11: Genesis is valid (self-referentially or with first child)
    #[tokio::test]
    async fn test_genesis_is_valid_self_referentially() {
        let app = EmptyBlockApp::new();
        let genesis = app.genesis().await;
        let first_child = EmptyBlock::new(1, CoreBlock::id(&genesis));
        // Verify first child against genesis
        assert!(app.verify(&genesis, &first_child).await.is_ok());
    }
}
