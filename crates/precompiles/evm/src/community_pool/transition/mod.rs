//! Pure community-pool state transitions and effect planning.

mod post_block_accounting;

pub use post_block_accounting::{
    build_post_block_accounting_effect, CommunityPoolUnlockEffect, CommunityPoolUnlockState,
    PostBlockAccountingEffect, PostBlockAccountingEffectError, PostBlockAccountingInputs,
    PostBlockAccountingOutcome,
};
