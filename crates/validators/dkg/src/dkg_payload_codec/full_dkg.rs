use crate::dkg_payload_codec::{
    take_slice, take_u32, take_u64, DkgPayloadError, MAX_FULL_DKG_KEYS,
    MAX_FULL_DKG_POLYNOMIAL_BYTES,
};
use crate::{FullDkgOutputV1, FullDkgV1};

pub fn encode_full_dkg_v1(full_dkg: &FullDkgV1) -> Result<Vec<u8>, DkgPayloadError> {
    if full_dkg.output.dealers.len() > MAX_FULL_DKG_KEYS {
        return Err(DkgPayloadError::TooManyDealers {
            found: full_dkg.output.dealers.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.players.len() > MAX_FULL_DKG_KEYS {
        return Err(DkgPayloadError::TooManyPlayers {
            found: full_dkg.output.players.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.public_polynomial.len() > MAX_FULL_DKG_POLYNOMIAL_BYTES {
        return Err(DkgPayloadError::FullDkgPolynomialTooLarge {
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

pub fn decode_full_dkg_v1(bytes: &[u8]) -> Result<FullDkgV1, DkgPayloadError> {
    let mut cursor = bytes;
    let epoch = take_u64(&mut cursor, "full_dkg.epoch")?;

    let dealers_len = take_u32(&mut cursor, "full_dkg.dealers_len")? as usize;
    if dealers_len > MAX_FULL_DKG_KEYS {
        return Err(DkgPayloadError::TooManyDealers {
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
        return Err(DkgPayloadError::TooManyPlayers {
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
        return Err(DkgPayloadError::FullDkgPolynomialTooLarge {
            found: polynomial_len,
            max: MAX_FULL_DKG_POLYNOMIAL_BYTES,
        });
    }
    let public_polynomial =
        take_slice(&mut cursor, polynomial_len, "full_dkg.public_polynomial")?.to_vec();

    if !cursor.is_empty() {
        return Err(DkgPayloadError::UnexpectedTrailingBytes);
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
