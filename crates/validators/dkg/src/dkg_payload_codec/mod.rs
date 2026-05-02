mod full_dkg;
mod reshare;

use bytes::Buf;

pub use full_dkg::{decode_full_dkg_v1, encode_full_dkg_v1};
pub use reshare::{decode_reshare_v1, encode_reshare_v1};

const MAX_FULL_DKG_KEYS: usize = 1024;
const MAX_RESHARE_KEYS: usize = 1024;
const MAX_FULL_DKG_POLYNOMIAL_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DkgPayloadError {
    UnexpectedTrailingBytes,
    Truncated(&'static str),
    TooManyDealers { found: usize, max: usize },
    TooManyPlayers { found: usize, max: usize },
    TooManyResharePlayers { found: usize, max: usize },
    FullDkgPolynomialTooLarge { found: usize, max: usize },
}

impl std::fmt::Display for DkgPayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedTrailingBytes => write!(f, "unexpected trailing bytes in DKG payload"),
            Self::Truncated(ctx) => write!(f, "truncated DKG payload while reading {ctx}"),
            Self::TooManyDealers { found, max } => {
                write!(f, "too many full_dkg dealers: found {found}, max {max}")
            }
            Self::TooManyPlayers { found, max } => {
                write!(f, "too many full_dkg players: found {found}, max {max}")
            }
            Self::TooManyResharePlayers { found, max } => {
                write!(f, "too many reshare players: found {found}, max {max}")
            }
            Self::FullDkgPolynomialTooLarge { found, max } => write!(
                f,
                "full_dkg public_polynomial too large: found {found}, max {max}"
            ),
        }
    }
}

impl std::error::Error for DkgPayloadError {}

fn take_slice<'a>(
    cursor: &mut &'a [u8],
    len: usize,
    ctx: &'static str,
) -> Result<&'a [u8], DkgPayloadError> {
    if cursor.len() < len {
        return Err(DkgPayloadError::Truncated(ctx));
    }
    let (head, tail) = cursor.split_at(len);
    *cursor = tail;
    Ok(head)
}

fn take_u32(cursor: &mut &[u8], ctx: &'static str) -> Result<u32, DkgPayloadError> {
    if cursor.len() < 4 {
        return Err(DkgPayloadError::Truncated(ctx));
    }
    Ok(cursor.get_u32_le())
}

fn take_u64(cursor: &mut &[u8], ctx: &'static str) -> Result<u64, DkgPayloadError> {
    if cursor.len() < 8 {
        return Err(DkgPayloadError::Truncated(ctx));
    }
    Ok(cursor.get_u64_le())
}
