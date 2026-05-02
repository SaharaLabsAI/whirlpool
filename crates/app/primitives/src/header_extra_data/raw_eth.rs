use crate::header_extra_data::{
    decode_header_extra_data, CanonicalHeaderExtraDataV1, HeaderExtraDataError,
    PROPOSER_PUBLIC_KEY_LEN,
};

pub fn proposer_public_key_from_raw_eth_section(
    decoded: &CanonicalHeaderExtraDataV1,
) -> Result<[u8; 32], HeaderExtraDataError> {
    let Some(raw_eth) = decoded.raw_eth.as_ref() else {
        return Err(HeaderExtraDataError::MissingRawEth);
    };
    proposer_public_key_from_raw_eth(raw_eth)
}

pub fn proposer_public_key_from_extra_data(bytes: &[u8]) -> Result<[u8; 32], HeaderExtraDataError> {
    let decoded = decode_header_extra_data(bytes)?;
    proposer_public_key_from_raw_eth_section(&decoded)
}

pub fn project_raw_eth_extra_data(bytes: &[u8]) -> Vec<u8> {
    decode_header_extra_data(bytes)
        .ok()
        .and_then(|decoded| decoded.raw_eth)
        .unwrap_or_default()
}

fn proposer_public_key_from_raw_eth(raw_eth: &[u8]) -> Result<[u8; 32], HeaderExtraDataError> {
    if raw_eth.len() != PROPOSER_PUBLIC_KEY_LEN {
        return Err(HeaderExtraDataError::InvalidRawEthLen {
            found: raw_eth.len(),
        });
    }
    let mut proposer_public_key = [0u8; 32];
    proposer_public_key.copy_from_slice(raw_eth);
    Ok(proposer_public_key)
}
