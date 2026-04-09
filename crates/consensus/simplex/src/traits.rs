//! CommonwareBlock super-trait combining consensus-core and commonware-consensus block interfaces.

use commonware_consensus::Block as VendorBlock;
use consensus::traits::Block as CoreBlock;

/// A block type that satisfies both the consensus-core `Block` trait
/// and the commonware-consensus `Block` trait requirements.
///
/// Any concrete block type used with the Commonware adapter must implement
/// both sets of block interfaces. This trait is automatically implemented
/// for any type that satisfies both constraints via a blanket implementation.
pub trait CommonwareBlock: CoreBlock + VendorBlock + Clone {}

/// Blanket implementation: any type implementing both `CoreBlock` and `VendorBlock`
/// plus `Clone` automatically satisfies `CommonwareBlock`.
impl<T> CommonwareBlock for T where T: CoreBlock + VendorBlock + Clone {}
