use std::time::Duration;

pub const NAMESPACE: &[u8] = b"sahara-chain-v0";
pub const BLOCK_INTERVAL: Duration = Duration::from_secs(5);
pub const BIND_ADDR: &str = "127.0.0.1:0";
pub const VALIDATOR_SEED: u64 = 0;
pub const RPC_BIND_ADDR: &str = "127.0.0.1:8545";
