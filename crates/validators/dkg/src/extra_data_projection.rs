use super::{decode_extra_data, ExtraDataError, RAW_ETH_PROPOSER_PUBLIC_KEY_LEN};

pub fn proposer_public_key_from_extra_data(extra_data: &[u8]) -> Option<[u8; 32]> {
    let decoded = decode_extra_data(extra_data).ok()?;
    let raw_eth = decoded.raw_eth?;
    proposer_public_key_from_raw_eth(&raw_eth).ok()
}

pub fn project_raw_eth_extra_data(extra_data: &[u8]) -> Vec<u8> {
    decode_extra_data(extra_data)
        .ok()
        .and_then(|decoded| decoded.raw_eth)
        .unwrap_or_default()
}

fn proposer_public_key_from_raw_eth(raw_eth: &[u8]) -> Result<[u8; 32], ExtraDataError> {
    if raw_eth.len() != RAW_ETH_PROPOSER_PUBLIC_KEY_LEN {
        return Err(ExtraDataError::InvalidRawEthLen {
            found: raw_eth.len(),
        });
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(raw_eth);
    Ok(out)
}
