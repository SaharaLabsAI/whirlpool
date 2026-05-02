use bytes::Buf;
use validators_dkg::{
    decode_full_dkg_v1, decode_reshare_v1, encode_full_dkg_v1, encode_reshare_v1,
};

use crate::header_extra_data::{
    CanonicalHeaderExtraDataV1, DkgHeaderSections, HeaderExtraDataError, HEADER_EXTRA_DATA_MAGIC,
    HEADER_EXTRA_DATA_SECTION_FULL_DKG_V1, HEADER_EXTRA_DATA_SECTION_RAW_ETH,
    HEADER_EXTRA_DATA_SECTION_RESHARE_V1, HEADER_EXTRA_DATA_VERSION, MAX_RAW_ETH_EXTRA_DATA_BYTES,
    MAX_TOTAL_HEADER_EXTRA_DATA_BYTES,
};

pub fn build_header_extra_data(
    proposer_public_key: [u8; 32],
    dkg: DkgHeaderSections,
) -> Result<Vec<u8>, HeaderExtraDataError> {
    encode_header_extra_data(&CanonicalHeaderExtraDataV1 {
        raw_eth: Some(proposer_public_key.to_vec()),
        dkg,
    })
}

pub fn encode_header_extra_data(
    data: &CanonicalHeaderExtraDataV1,
) -> Result<Vec<u8>, HeaderExtraDataError> {
    let mut sections: Vec<(u8, Vec<u8>)> = Vec::new();

    if let Some(raw_eth) = &data.raw_eth {
        if raw_eth.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
            return Err(HeaderExtraDataError::RawEthTooLarge {
                found: raw_eth.len(),
                max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
            });
        }
        sections.push((HEADER_EXTRA_DATA_SECTION_RAW_ETH, raw_eth.clone()));
    }

    if let Some(full_dkg) = &data.dkg.full_dkg {
        let full_dkg_payload = encode_full_dkg_v1(full_dkg)?;
        sections.push((HEADER_EXTRA_DATA_SECTION_FULL_DKG_V1, full_dkg_payload));
    }

    if let Some(reshare) = &data.dkg.reshare {
        if data.dkg.full_dkg.is_none() {
            return Err(HeaderExtraDataError::InvalidSectionOrder {
                section: HEADER_EXTRA_DATA_SECTION_RESHARE_V1,
            });
        }
        let reshare_payload = encode_reshare_v1(reshare)?;
        sections.push((HEADER_EXTRA_DATA_SECTION_RESHARE_V1, reshare_payload));
    }

    if sections.is_empty() {
        return Err(HeaderExtraDataError::EmptySections);
    }

    let mut out = Vec::new();
    out.extend_from_slice(HEADER_EXTRA_DATA_MAGIC);
    out.push(HEADER_EXTRA_DATA_VERSION);
    out.push(u8::try_from(sections.len()).expect("section count fits into u8"));

    for (section, payload) in sections {
        out.push(section);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
    }

    if out.len() > MAX_TOTAL_HEADER_EXTRA_DATA_BYTES {
        return Err(HeaderExtraDataError::TotalExtraDataTooLarge {
            found: out.len(),
            max: MAX_TOTAL_HEADER_EXTRA_DATA_BYTES,
        });
    }

    Ok(out)
}

pub fn decode_header_extra_data(
    bytes: &[u8],
) -> Result<CanonicalHeaderExtraDataV1, HeaderExtraDataError> {
    if !bytes.starts_with(HEADER_EXTRA_DATA_MAGIC) {
        return Err(HeaderExtraDataError::InvalidMagic);
    }
    decode_enveloped_header_extra_data(bytes)
}

fn decode_enveloped_header_extra_data(
    bytes: &[u8],
) -> Result<CanonicalHeaderExtraDataV1, HeaderExtraDataError> {
    if bytes.len() > MAX_TOTAL_HEADER_EXTRA_DATA_BYTES {
        return Err(HeaderExtraDataError::TotalExtraDataTooLarge {
            found: bytes.len(),
            max: MAX_TOTAL_HEADER_EXTRA_DATA_BYTES,
        });
    }

    let mut cursor = bytes;
    let magic = take_slice(&mut cursor, 4, "magic")?;
    if magic != HEADER_EXTRA_DATA_MAGIC {
        return Err(HeaderExtraDataError::InvalidMagic);
    }

    let version = take_u8(&mut cursor, "version")?;
    if version != HEADER_EXTRA_DATA_VERSION {
        return Err(HeaderExtraDataError::UnsupportedVersion { found: version });
    }

    let section_count = take_u8(&mut cursor, "section_count")? as usize;
    let mut raw_eth = None;
    let mut full_dkg = None;
    let mut reshare = None;

    for _ in 0..section_count {
        let section = take_u8(&mut cursor, "section id")?;
        let section_len = take_u32(&mut cursor, "section length")? as usize;

        if section_len > MAX_TOTAL_HEADER_EXTRA_DATA_BYTES {
            return Err(HeaderExtraDataError::SectionTooLarge {
                section,
                found: section_len,
                max: MAX_TOTAL_HEADER_EXTRA_DATA_BYTES,
            });
        }

        let payload = take_slice(&mut cursor, section_len, "section payload")?;

        match section {
            HEADER_EXTRA_DATA_SECTION_RAW_ETH => {
                decode_raw_eth_section(section, payload, &mut raw_eth, full_dkg.as_ref())?
            }
            HEADER_EXTRA_DATA_SECTION_FULL_DKG_V1 => {
                decode_full_dkg_section(section, payload, &mut full_dkg, reshare.as_ref())?
            }
            HEADER_EXTRA_DATA_SECTION_RESHARE_V1 => {
                decode_reshare_section(section, payload, &mut reshare, full_dkg.as_ref())?
            }
            _ => return Err(HeaderExtraDataError::UnknownSection { section }),
        }
    }

    if !cursor.is_empty() {
        return Err(HeaderExtraDataError::UnexpectedTrailingBytes);
    }

    if raw_eth.is_none() && full_dkg.is_none() && reshare.is_none() {
        return Err(HeaderExtraDataError::EmptySections);
    }

    Ok(CanonicalHeaderExtraDataV1 {
        raw_eth,
        dkg: DkgHeaderSections { full_dkg, reshare },
    })
}

fn decode_raw_eth_section(
    section: u8,
    payload: &[u8],
    raw_eth: &mut Option<Vec<u8>>,
    full_dkg: Option<&validators_dkg::FullDkgV1>,
) -> Result<(), HeaderExtraDataError> {
    if full_dkg.is_some() {
        return Err(HeaderExtraDataError::InvalidSectionOrder { section });
    }
    if raw_eth.is_some() {
        return Err(HeaderExtraDataError::DuplicateSection { section });
    }
    if payload.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
        return Err(HeaderExtraDataError::RawEthTooLarge {
            found: payload.len(),
            max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
        });
    }

    *raw_eth = Some(payload.to_vec());
    Ok(())
}

fn decode_full_dkg_section(
    section: u8,
    payload: &[u8],
    full_dkg: &mut Option<validators_dkg::FullDkgV1>,
    reshare: Option<&validators_dkg::ReshareV1>,
) -> Result<(), HeaderExtraDataError> {
    if full_dkg.is_some() {
        return Err(HeaderExtraDataError::DuplicateSection { section });
    }
    if reshare.is_some() {
        return Err(HeaderExtraDataError::InvalidSectionOrder { section });
    }

    *full_dkg = Some(decode_full_dkg_v1(payload)?);
    Ok(())
}

fn decode_reshare_section(
    section: u8,
    payload: &[u8],
    reshare: &mut Option<validators_dkg::ReshareV1>,
    full_dkg: Option<&validators_dkg::FullDkgV1>,
) -> Result<(), HeaderExtraDataError> {
    if reshare.is_some() {
        return Err(HeaderExtraDataError::DuplicateSection { section });
    }
    if full_dkg.is_none() {
        return Err(HeaderExtraDataError::InvalidSectionOrder { section });
    }

    *reshare = Some(decode_reshare_v1(payload)?);
    Ok(())
}

fn take_slice<'a>(
    cursor: &mut &'a [u8],
    len: usize,
    ctx: &'static str,
) -> Result<&'a [u8], HeaderExtraDataError> {
    if cursor.len() < len {
        return Err(HeaderExtraDataError::Truncated(ctx));
    }
    let (head, tail) = cursor.split_at(len);
    *cursor = tail;
    Ok(head)
}

fn take_u8(cursor: &mut &[u8], ctx: &'static str) -> Result<u8, HeaderExtraDataError> {
    if cursor.is_empty() {
        return Err(HeaderExtraDataError::Truncated(ctx));
    }
    Ok(cursor.get_u8())
}

fn take_u32(cursor: &mut &[u8], ctx: &'static str) -> Result<u32, HeaderExtraDataError> {
    if cursor.len() < 4 {
        return Err(HeaderExtraDataError::Truncated(ctx));
    }
    Ok(cursor.get_u32_le())
}
