use std::fmt::Debug;
use std::hash::Hash;

/// Core block abstraction for the consensus layer.
pub trait Block: Send + Sync + 'static {
    /// Unique identifier for a block (e.g., a hash).
    type Id: Copy + Eq + Hash + Debug + Send + Sync + 'static;

    /// Returns this block's unique identifier.
    fn id(&self) -> Self::Id;

    /// Returns the identifier of this block's parent.
    fn parent_id(&self) -> Self::Id;

    /// Returns the height (slot number) of this block.
    fn height(&self) -> u64;
}
