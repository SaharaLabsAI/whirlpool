use validators_dkg::{
    DkgHeaderDecision, DkgHeaderSectionsRef, DkgPayloadError, FullDkgV1, ReshareV1,
};

mod codec;
mod history;
mod raw_eth;

pub use codec::{build_header_extra_data, decode_header_extra_data, encode_header_extra_data};
pub use history::HeaderExtraDataHistory;
pub use raw_eth::{
    project_raw_eth_extra_data, proposer_public_key_from_extra_data,
    proposer_public_key_from_raw_eth_section,
};

const HEADER_EXTRA_DATA_MAGIC: &[u8; 4] = b"WDX1";
const HEADER_EXTRA_DATA_VERSION: u8 = 1;
const HEADER_EXTRA_DATA_SECTION_RAW_ETH: u8 = 1;
const HEADER_EXTRA_DATA_SECTION_FULL_DKG_V1: u8 = 2;
const HEADER_EXTRA_DATA_SECTION_RESHARE_V1: u8 = 3;
const MAX_TOTAL_HEADER_EXTRA_DATA_BYTES: usize = 256 * 1024;
const MAX_RAW_ETH_EXTRA_DATA_BYTES: usize = 1024;
const PROPOSER_PUBLIC_KEY_LEN: usize = 32;

pub type RawEthProposerCarrier = Vec<u8>;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DkgHeaderSections {
    pub full_dkg: Option<FullDkgV1>,
    pub reshare: Option<ReshareV1>,
}

impl DkgHeaderSections {
    pub fn as_ref(&self) -> DkgHeaderSectionsRef<'_> {
        DkgHeaderSectionsRef {
            full_dkg: self.full_dkg.as_ref(),
            reshare: self.reshare.as_ref(),
        }
    }
}

impl From<DkgHeaderDecision> for DkgHeaderSections {
    fn from(decision: DkgHeaderDecision) -> Self {
        Self {
            full_dkg: decision.full_dkg,
            reshare: decision.reshare,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CanonicalHeaderExtraDataV1 {
    pub raw_eth: Option<RawEthProposerCarrier>,
    pub dkg: DkgHeaderSections,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeaderExtraDataError {
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
    MissingRawEth,
    TotalExtraDataTooLarge {
        found: usize,
        max: usize,
    },
    SectionTooLarge {
        section: u8,
        found: usize,
        max: usize,
    },
    DkgPayload(DkgPayloadError),
}

impl std::fmt::Display for HeaderExtraDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySections => {
                write!(
                    f,
                    "canonical header extra_data must contain at least one section"
                )
            }
            Self::InvalidMagic => write!(f, "invalid canonical header extra_data magic"),
            Self::UnsupportedVersion { found } => {
                write!(
                    f,
                    "unsupported canonical header extra_data version: {found}"
                )
            }
            Self::UnexpectedTrailingBytes => {
                write!(
                    f,
                    "unexpected trailing bytes in canonical header extra_data"
                )
            }
            Self::Truncated(ctx) => {
                write!(
                    f,
                    "truncated canonical header extra_data while reading {ctx}"
                )
            }
            Self::DuplicateSection { section } => {
                write!(
                    f,
                    "duplicate canonical header extra_data section: {section}"
                )
            }
            Self::InvalidSectionOrder { section } => write!(
                f,
                "invalid canonical header extra_data section order at section: {section}"
            ),
            Self::UnknownSection { section } => {
                write!(f, "unknown canonical header extra_data section: {section}")
            }
            Self::RawEthTooLarge { found, max } => {
                write!(f, "raw_eth section too large: found {found}, max {max}")
            }
            Self::InvalidRawEthLen { found } => write!(
                f,
                "raw_eth proposer key must be {PROPOSER_PUBLIC_KEY_LEN} bytes, found {found}"
            ),
            Self::MissingRawEth => {
                write!(f, "missing raw_eth section in canonical header extra_data")
            }
            Self::TotalExtraDataTooLarge { found, max } => write!(
                f,
                "canonical header extra_data too large: found {found}, max {max}"
            ),
            Self::SectionTooLarge {
                section,
                found,
                max,
            } => write!(f, "section {section} too large: found {found}, max {max}"),
            Self::DkgPayload(source) => write!(f, "invalid DKG header payload: {source}"),
        }
    }
}

impl std::error::Error for HeaderExtraDataError {}

impl From<DkgPayloadError> for HeaderExtraDataError {
    fn from(source: DkgPayloadError) -> Self {
        Self::DkgPayload(source)
    }
}
