pub const BASE_VALIDATORS_GAS: u64 = 3_000;
pub const PER_VALIDATOR_GAS: u64 = 350;

pub fn validators_gas(entries: usize) -> u64 {
    BASE_VALIDATORS_GAS.saturating_add(PER_VALIDATOR_GAS.saturating_mul(entries as u64))
}
