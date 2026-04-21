use commonware_cryptography::sha256;
use std::fmt;

mod block;
mod block_traits;
mod extra_data_codec;
mod extra_data_projection;

#[cfg(test)]
mod tests;

pub use block::EvmBlock;
pub use extra_data_codec::{decode_extra_data, encode_canonical_extra_data};
pub use extra_data_projection::{
    legacy_proposer_extra_data_bytes, project_raw_eth_extra_data,
    proposer_public_key_from_extra_data,
};

pub type BlockId = [u8; 32];

type BlockDigest = sha256::Digest;

const EXTRA_DATA_MAGIC: &[u8; 4] = b"WDX1";
const EXTRA_DATA_VERSION: u8 = 1;
const EXTRA_DATA_SECTION_RAW_ETH: u8 = 1;
const EXTRA_DATA_SECTION_FULL_DKG_V1: u8 = 2;
const EXTRA_DATA_SECTION_RESHARE_V1: u8 = 3;
const MAX_TOTAL_EXTRA_DATA_BYTES: usize = 256 * 1024;
const MAX_RAW_ETH_EXTRA_DATA_BYTES: usize = 1024;
const MAX_FULL_DKG_KEYS: usize = 1024;
const MAX_RESHARE_KEYS: usize = 1024;
const MAX_FULL_DKG_POLYNOMIAL_BYTES: usize = 128 * 1024;
pub const LEGACY_PROPOSER_EXTRA_DATA_LEN: usize = 32;

#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDkgOutputV1 {
    pub dealers: Vec<[u8; 32]>,
    pub players: Vec<[u8; 32]>,
    pub public_polynomial: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDkgV1 {
    pub epoch: u64,
    pub output: FullDkgOutputV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReshareV1 {
    pub target_epoch: u64,
    pub players: Vec<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CanonicalExtraDataV1 {
    pub raw_eth: Option<Vec<u8>>,
    pub full_dkg: Option<FullDkgV1>,
    pub reshare: Option<ReshareV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraDataDecodeMode {
    Strict,
    Legacy,
    RpcProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtraDataError {
    EmptySections,
    InvalidMagic,
    UnsupportedVersion {
        found: u8,
    },
    UnexpectedTrailingBytes,
    Truncated(&'static str),
    DuplicateSection {
        section: u8,
    },
    InvalidSectionOrder {
        section: u8,
    },
    UnknownSection {
        section: u8,
    },
    RawEthTooLarge {
        found: usize,
        max: usize,
    },
    InvalidRawEthLen {
        found: usize,
    },
    LegacyUnsupportedLen {
        found: usize,
    },
    TooManyDealers {
        found: usize,
        max: usize,
    },
    TooManyPlayers {
        found: usize,
        max: usize,
    },
    TooManyResharePlayers {
        found: usize,
        max: usize,
    },
    FullDkgPolynomialTooLarge {
        found: usize,
        max: usize,
    },
    TotalExtraDataTooLarge {
        found: usize,
        max: usize,
    },
    SectionTooLarge {
        section: u8,
        found: usize,
        max: usize,
    },
}

impl fmt::Display for ExtraDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySections => write!(f, "canonical extra_data must contain at least one section"),
            Self::InvalidMagic => write!(f, "invalid canonical extra_data magic"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported canonical extra_data version: {found}")
            }
            Self::UnexpectedTrailingBytes => write!(f, "unexpected trailing bytes in canonical extra_data"),
            Self::Truncated(ctx) => write!(f, "truncated canonical extra_data while reading {ctx}"),
            Self::DuplicateSection { section } => {
                write!(f, "duplicate canonical extra_data section: {section}")
            }
            Self::InvalidSectionOrder { section } => {
                write!(f, "invalid canonical extra_data section order at section: {section}")
            }
            Self::UnknownSection { section } => {
                write!(f, "unknown canonical extra_data section: {section}")
            }
            Self::RawEthTooLarge { found, max } => {
                write!(f, "raw_eth section too large: found {found}, max {max}")
            }
            Self::InvalidRawEthLen { found } => write!(
                f,
                "raw_eth proposer key must be {LEGACY_PROPOSER_EXTRA_DATA_LEN} bytes, found {found}"
            ),
            Self::LegacyUnsupportedLen { found } => write!(
                f,
                "legacy extra_data must be exactly {LEGACY_PROPOSER_EXTRA_DATA_LEN} bytes, found {found}"
            ),
            Self::TooManyDealers { found, max } => {
                write!(f, "too many full_dkg dealers: found {found}, max {max}")
            }
            Self::TooManyPlayers { found, max } => {
                write!(f, "too many full_dkg players: found {found}, max {max}")
            }
            Self::TooManyResharePlayers { found, max } => {
                write!(f, "too many reshare players: found {found}, max {max}")
            }
            Self::FullDkgPolynomialTooLarge { found, max } => {
                write!(f, "full_dkg public_polynomial too large: found {found}, max {max}")
            }
            Self::TotalExtraDataTooLarge { found, max } => {
                write!(f, "canonical extra_data too large: found {found}, max {max}")
            }
            Self::SectionTooLarge {
                section,
                found,
                max,
            } => {
                write!(f, "section {section} too large: found {found}, max {max}")
            }
        }
    }
}

impl std::error::Error for ExtraDataError {}
