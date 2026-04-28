use bytes::Buf;

use super::{
    CanonicalExtraDataV1, ExtraDataError, FullDkgOutputV1, FullDkgV1, ReshareV1, EXTRA_DATA_MAGIC,
    EXTRA_DATA_SECTION_FULL_DKG_V1, EXTRA_DATA_SECTION_RAW_ETH, EXTRA_DATA_SECTION_RESHARE_V1,
    EXTRA_DATA_VERSION, MAX_FULL_DKG_KEYS, MAX_FULL_DKG_POLYNOMIAL_BYTES,
    MAX_RAW_ETH_EXTRA_DATA_BYTES, MAX_RESHARE_KEYS, MAX_TOTAL_EXTRA_DATA_BYTES,
};

pub fn encode_canonical_extra_data(data: &CanonicalExtraDataV1) -> Result<Vec<u8>, ExtraDataError> {
    let mut sections: Vec<(u8, Vec<u8>)> = Vec::new();

    if let Some(raw_eth) = &data.raw_eth {
        if raw_eth.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
            return Err(ExtraDataError::RawEthTooLarge {
                found: raw_eth.len(),
                max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
            });
        }
        sections.push((EXTRA_DATA_SECTION_RAW_ETH, raw_eth.clone()));
    }

    if let Some(full_dkg) = &data.full_dkg {
        let full_dkg_payload = encode_full_dkg_v1(full_dkg)?;
        sections.push((EXTRA_DATA_SECTION_FULL_DKG_V1, full_dkg_payload));
    }

    if let Some(reshare) = &data.reshare {
        if data.full_dkg.is_none() {
            return Err(ExtraDataError::InvalidSectionOrder {
                section: EXTRA_DATA_SECTION_RESHARE_V1,
            });
        }
        let reshare_payload = encode_reshare_v1(reshare)?;
        sections.push((EXTRA_DATA_SECTION_RESHARE_V1, reshare_payload));
    }

    if sections.is_empty() {
        return Err(ExtraDataError::EmptySections);
    }

    let mut out = Vec::new();
    out.extend_from_slice(EXTRA_DATA_MAGIC);
    out.push(EXTRA_DATA_VERSION);
    out.push(u8::try_from(sections.len()).expect("section count fits into u8"));

    for (section, payload) in sections {
        out.push(section);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
    }

    if out.len() > MAX_TOTAL_EXTRA_DATA_BYTES {
        return Err(ExtraDataError::TotalExtraDataTooLarge {
            found: out.len(),
            max: MAX_TOTAL_EXTRA_DATA_BYTES,
        });
    }

    Ok(out)
}

pub fn decode_extra_data(bytes: &[u8]) -> Result<CanonicalExtraDataV1, ExtraDataError> {
    if !bytes.starts_with(EXTRA_DATA_MAGIC) {
        return Err(ExtraDataError::InvalidMagic);
    }
    decode_enveloped_extra_data(bytes)
}

fn decode_enveloped_extra_data(bytes: &[u8]) -> Result<CanonicalExtraDataV1, ExtraDataError> {
    if bytes.len() > MAX_TOTAL_EXTRA_DATA_BYTES {
        return Err(ExtraDataError::TotalExtraDataTooLarge {
            found: bytes.len(),
            max: MAX_TOTAL_EXTRA_DATA_BYTES,
        });
    }

    let mut cursor = bytes;
    let magic = take_slice(&mut cursor, 4, "magic")?;
    if magic != EXTRA_DATA_MAGIC {
        return Err(ExtraDataError::InvalidMagic);
    }

    let version = take_u8(&mut cursor, "version")?;
    if version != EXTRA_DATA_VERSION {
        return Err(ExtraDataError::UnsupportedVersion { found: version });
    }

    let section_count = take_u8(&mut cursor, "section_count")? as usize;
    let mut raw_eth = None;
    let mut full_dkg = None;
    let mut reshare = None;

    for _ in 0..section_count {
        let section = take_u8(&mut cursor, "section id")?;
        let section_len = take_u32(&mut cursor, "section length")? as usize;

        if section_len > MAX_TOTAL_EXTRA_DATA_BYTES {
            return Err(ExtraDataError::SectionTooLarge {
                section,
                found: section_len,
                max: MAX_TOTAL_EXTRA_DATA_BYTES,
            });
        }

        let payload = take_slice(&mut cursor, section_len, "section payload")?;

        match section {
            EXTRA_DATA_SECTION_RAW_ETH => {
                if full_dkg.is_some() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                if raw_eth.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if payload.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
                    return Err(ExtraDataError::RawEthTooLarge {
                        found: payload.len(),
                        max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
                    });
                }
                raw_eth = Some(payload.to_vec());
            }
            EXTRA_DATA_SECTION_FULL_DKG_V1 => {
                if full_dkg.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if reshare.is_some() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                full_dkg = Some(decode_full_dkg_v1(payload)?);
            }
            EXTRA_DATA_SECTION_RESHARE_V1 => {
                if reshare.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if full_dkg.is_none() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                reshare = Some(decode_reshare_v1(payload)?);
            }
            _ => return Err(ExtraDataError::UnknownSection { section }),
        }
    }

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    if raw_eth.is_none() && full_dkg.is_none() && reshare.is_none() {
        return Err(ExtraDataError::EmptySections);
    }

    Ok(CanonicalExtraDataV1 {
        raw_eth,
        full_dkg,
        reshare,
    })
}

fn encode_full_dkg_v1(full_dkg: &FullDkgV1) -> Result<Vec<u8>, ExtraDataError> {
    if full_dkg.output.dealers.len() > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyDealers {
            found: full_dkg.output.dealers.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.players.len() > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyPlayers {
            found: full_dkg.output.players.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.public_polynomial.len() > MAX_FULL_DKG_POLYNOMIAL_BYTES {
        return Err(ExtraDataError::FullDkgPolynomialTooLarge {
            found: full_dkg.output.public_polynomial.len(),
            max: MAX_FULL_DKG_POLYNOMIAL_BYTES,
        });
    }

    let mut out = Vec::new();
    out.extend_from_slice(&full_dkg.epoch.to_le_bytes());
    out.extend_from_slice(&(full_dkg.output.dealers.len() as u32).to_le_bytes());
    for dealer in &full_dkg.output.dealers {
        out.extend_from_slice(dealer);
    }
    out.extend_from_slice(&(full_dkg.output.players.len() as u32).to_le_bytes());
    for player in &full_dkg.output.players {
        out.extend_from_slice(player);
    }
    out.extend_from_slice(&(full_dkg.output.public_polynomial.len() as u32).to_le_bytes());
    out.extend_from_slice(&full_dkg.output.public_polynomial);
    Ok(out)
}

fn decode_full_dkg_v1(bytes: &[u8]) -> Result<FullDkgV1, ExtraDataError> {
    let mut cursor = bytes;
    let epoch = take_u64(&mut cursor, "full_dkg.epoch")?;

    let dealers_len = take_u32(&mut cursor, "full_dkg.dealers_len")? as usize;
    if dealers_len > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyDealers {
            found: dealers_len,
            max: MAX_FULL_DKG_KEYS,
        });
    }

    let mut dealers = Vec::with_capacity(dealers_len);
    for _ in 0..dealers_len {
        let dealer = take_slice(&mut cursor, 32, "full_dkg.dealer")?;
        let mut dealer_key = [0u8; 32];
        dealer_key.copy_from_slice(dealer);
        dealers.push(dealer_key);
    }

    let players_len = take_u32(&mut cursor, "full_dkg.players_len")? as usize;
    if players_len > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyPlayers {
            found: players_len,
            max: MAX_FULL_DKG_KEYS,
        });
    }

    let mut players = Vec::with_capacity(players_len);
    for _ in 0..players_len {
        let player = take_slice(&mut cursor, 32, "full_dkg.player")?;
        let mut player_key = [0u8; 32];
        player_key.copy_from_slice(player);
        players.push(player_key);
    }

    let polynomial_len = take_u32(&mut cursor, "full_dkg.public_polynomial_len")? as usize;
    if polynomial_len > MAX_FULL_DKG_POLYNOMIAL_BYTES {
        return Err(ExtraDataError::FullDkgPolynomialTooLarge {
            found: polynomial_len,
            max: MAX_FULL_DKG_POLYNOMIAL_BYTES,
        });
    }
    let public_polynomial =
        take_slice(&mut cursor, polynomial_len, "full_dkg.public_polynomial")?.to_vec();

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    Ok(FullDkgV1 {
        epoch,
        output: FullDkgOutputV1 {
            dealers,
            players,
            public_polynomial,
        },
    })
}

fn encode_reshare_v1(reshare: &ReshareV1) -> Result<Vec<u8>, ExtraDataError> {
    if reshare.players.len() > MAX_RESHARE_KEYS {
        return Err(ExtraDataError::TooManyResharePlayers {
            found: reshare.players.len(),
            max: MAX_RESHARE_KEYS,
        });
    }

    let mut out = Vec::new();
    out.extend_from_slice(&reshare.target_epoch.to_le_bytes());
    out.extend_from_slice(&(reshare.players.len() as u32).to_le_bytes());
    for player in &reshare.players {
        out.extend_from_slice(player);
    }
    Ok(out)
}

fn decode_reshare_v1(bytes: &[u8]) -> Result<ReshareV1, ExtraDataError> {
    let mut cursor = bytes;
    let target_epoch = take_u64(&mut cursor, "reshare.target_epoch")?;

    let players_len = take_u32(&mut cursor, "reshare.players_len")? as usize;
    if players_len > MAX_RESHARE_KEYS {
        return Err(ExtraDataError::TooManyResharePlayers {
            found: players_len,
            max: MAX_RESHARE_KEYS,
        });
    }

    let mut players = Vec::with_capacity(players_len);
    for _ in 0..players_len {
        let player = take_slice(&mut cursor, 32, "reshare.player")?;
        let mut player_key = [0u8; 32];
        player_key.copy_from_slice(player);
        players.push(player_key);
    }

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    Ok(ReshareV1 {
        target_epoch,
        players,
    })
}

fn take_slice<'a>(
    cursor: &mut &'a [u8],
    len: usize,
    label: &'static str,
) -> Result<&'a [u8], ExtraDataError> {
    if cursor.remaining() < len {
        return Err(ExtraDataError::Truncated(label));
    }
    let out = &cursor[..len];
    *cursor = &cursor[len..];
    Ok(out)
}

fn take_u8(cursor: &mut &[u8], label: &'static str) -> Result<u8, ExtraDataError> {
    Ok(*take_slice(cursor, 1, label)?
        .first()
        .expect("slice len checked"))
}

fn take_u32(cursor: &mut &[u8], label: &'static str) -> Result<u32, ExtraDataError> {
    let bytes = take_slice(cursor, 4, label)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(out))
}

fn take_u64(cursor: &mut &[u8], label: &'static str) -> Result<u64, ExtraDataError> {
    let bytes = take_slice(cursor, 8, label)?;
    let mut out = [0u8; 8];
    out.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(out))
}
