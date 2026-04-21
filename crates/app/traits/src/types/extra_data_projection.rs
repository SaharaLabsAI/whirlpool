use super::{decode_extra_data, ExtraDataDecodeMode, LEGACY_PROPOSER_EXTRA_DATA_LEN};

pub fn legacy_proposer_extra_data_bytes(proposer_public_key: [u8; 32]) -> Vec<u8> {
    proposer_public_key.to_vec()
}

pub fn proposer_public_key_from_extra_data(extra_data: &[u8]) -> Option<[u8; 32]> {
    let decoded = decode_extra_data(extra_data, ExtraDataDecodeMode::Legacy).ok()?;
    let raw_eth = decoded.raw_eth?;
    if raw_eth.len() != LEGACY_PROPOSER_EXTRA_DATA_LEN {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw_eth);
    Some(out)
}

pub fn project_raw_eth_extra_data(extra_data: &[u8]) -> Vec<u8> {
    match decode_extra_data(extra_data, ExtraDataDecodeMode::RpcProjection) {
        Ok(decoded) => decoded.raw_eth.unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}
