use crate::dkg_payload_codec::{take_slice, take_u32, take_u64, DkgPayloadError, MAX_RESHARE_KEYS};
use crate::ReshareV1;

pub fn encode_reshare_v1(reshare: &ReshareV1) -> Result<Vec<u8>, DkgPayloadError> {
    if reshare.players.len() > MAX_RESHARE_KEYS {
        return Err(DkgPayloadError::TooManyResharePlayers {
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

pub fn decode_reshare_v1(bytes: &[u8]) -> Result<ReshareV1, DkgPayloadError> {
    let mut cursor = bytes;
    let target_epoch = take_u64(&mut cursor, "reshare.target_epoch")?;

    let players_len = take_u32(&mut cursor, "reshare.players_len")? as usize;
    if players_len > MAX_RESHARE_KEYS {
        return Err(DkgPayloadError::TooManyResharePlayers {
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
        return Err(DkgPayloadError::UnexpectedTrailingBytes);
    }

    Ok(ReshareV1 {
        target_epoch,
        players,
    })
}
