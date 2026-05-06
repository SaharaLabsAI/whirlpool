mod dispatch;
mod output_epoch_start;
mod output_scalar;
mod read_calldata;
mod write_calldata;

pub use dispatch::{decode_call, EpochCall};
pub use output_epoch_start::{decode_epoch_blocks_output, decode_epoch_start_block_output};
pub use output_scalar::{
    decode_current_epoch_output, decode_next_epoch_block_output, decode_u64_output,
};
pub use read_calldata::{current_epoch_calldata, epoch_blocks_calldata, next_epoch_block_calldata};
pub use write_calldata::{
    advance_epoch_calldata, epoch_start_block_calldata, is_advance_epoch_calldata,
};
