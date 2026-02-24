use crate::block::Block;
use crate::error::ConsensusError;
use std::future::Future;

/// Application-level callbacks invoked by the consensus engine.
pub trait ConsensusApp: Send + Sync + 'static {
    /// The block type this application produces and validates.
    type Block: Block;

    /// Produce the genesis (initial) block.
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    /// Propose a new block building on `parent` at the given `height`.
    /// Returns `None` if the node should abstain from proposing.
    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Option<Self::Block>> + Send;

    /// Verify that `block` is valid given its `parent`.
    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<(), ConsensusError>> + Send;
}
