use validators_dkg::{
    decode_extra_data, encode_canonical_extra_data, CanonicalExtraDataV1, ExtraDataError,
};

const PROPOSER_PUBLIC_KEY_LEN: usize = 32;

pub fn build_raw_eth_envelope(proposer_public_key: [u8; 32]) -> Result<Vec<u8>, ExtraDataError> {
    encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(proposer_public_key.to_vec()),
        full_dkg: None,
        reshare: None,
    })
}

pub fn decode_strict_extra_data(bytes: &[u8]) -> Result<CanonicalExtraDataV1, ExtraDataError> {
    decode_extra_data(bytes)
}

pub fn proposer_public_key_from_raw_eth_section(
    decoded: &CanonicalExtraDataV1,
) -> Result<[u8; 32], ExtraDataError> {
    let Some(raw_eth) = decoded.raw_eth.as_ref() else {
        return Err(ExtraDataError::MissingRawEth);
    };
    proposer_public_key_from_raw_eth(raw_eth)
}

pub fn proposer_public_key_from_extra_data(bytes: &[u8]) -> Result<[u8; 32], ExtraDataError> {
    let decoded = decode_strict_extra_data(bytes)?;
    proposer_public_key_from_raw_eth_section(&decoded)
}

pub fn project_raw_eth_extra_data(bytes: &[u8]) -> Vec<u8> {
    validators_dkg::project_raw_eth_extra_data(bytes)
}

fn proposer_public_key_from_raw_eth(raw_eth: &[u8]) -> Result<[u8; 32], ExtraDataError> {
    if raw_eth.len() != PROPOSER_PUBLIC_KEY_LEN {
        return Err(ExtraDataError::InvalidRawEthLen {
            found: raw_eth.len(),
        });
    }
    let mut proposer_public_key = [0u8; 32];
    proposer_public_key.copy_from_slice(raw_eth);
    Ok(proposer_public_key)
}
