use commonware_cryptography::sha256;

mod block;
mod block_traits;
pub mod header_extra_data;

#[cfg(test)]
mod tests;

pub use alloy_consensus::Receipt;
pub use block::EvmBlock;

pub type BlockId = [u8; 32];

type BlockDigest = sha256::Digest;

#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}
