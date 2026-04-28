use std::collections::BTreeMap;
use std::fmt;

mod extra_data_codec;
mod extra_data_projection;

#[cfg(test)]
mod tests;

pub use extra_data_codec::{decode_extra_data, encode_canonical_extra_data};
pub use extra_data_projection::{project_raw_eth_extra_data, proposer_public_key_from_extra_data};

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
pub const RAW_ETH_PROPOSER_PUBLIC_KEY_LEN: usize = 32;

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
    MissingRawEth,
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
            Self::UnsupportedVersion { found } => write!(f, "unsupported canonical extra_data version: {found}"),
            Self::UnexpectedTrailingBytes => write!(f, "unexpected trailing bytes in canonical extra_data"),
            Self::Truncated(ctx) => write!(f, "truncated canonical extra_data while reading {ctx}"),
            Self::DuplicateSection { section } => write!(f, "duplicate canonical extra_data section: {section}"),
            Self::InvalidSectionOrder { section } => write!(f, "invalid canonical extra_data section order at section: {section}"),
            Self::UnknownSection { section } => write!(f, "unknown canonical extra_data section: {section}"),
            Self::RawEthTooLarge { found, max } => write!(f, "raw_eth section too large: found {found}, max {max}"),
            Self::InvalidRawEthLen { found } => write!(f, "raw_eth proposer key must be {RAW_ETH_PROPOSER_PUBLIC_KEY_LEN} bytes, found {found}"),
            Self::MissingRawEth => write!(f, "missing raw_eth section in canonical extra_data"),
            Self::TooManyDealers { found, max } => write!(f, "too many full_dkg dealers: found {found}, max {max}"),
            Self::TooManyPlayers { found, max } => write!(f, "too many full_dkg players: found {found}, max {max}"),
            Self::TooManyResharePlayers { found, max } => write!(f, "too many reshare players: found {found}, max {max}"),
            Self::FullDkgPolynomialTooLarge { found, max } => write!(f, "full_dkg public_polynomial too large: found {found}, max {max}"),
            Self::TotalExtraDataTooLarge { found, max } => write!(f, "canonical extra_data too large: found {found}, max {max}"),
            Self::SectionTooLarge { section, found, max } => write!(f, "section {section} too large: found {found}, max {max}"),
        }
    }
}

impl std::error::Error for ExtraDataError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochActivationTargets {
    pub boundary_epoch_e: u64,
    pub full_dkg_epoch: u64,
    pub reshare_target_epoch: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochActivationTargetError {
    #[error("full_dkg epoch overflow at boundary")]
    FullDkgEpochOverflow,
    #[error("reshare target epoch overflow at boundary")]
    ReshareTargetEpochOverflow,
}

impl EpochActivationTargets {
    pub fn from_post_advance_epoch(
        post_advance_epoch: u64,
    ) -> Result<Self, EpochActivationTargetError> {
        let full_dkg_epoch = post_advance_epoch
            .checked_add(1)
            .ok_or(EpochActivationTargetError::FullDkgEpochOverflow)?;
        let reshare_target_epoch = post_advance_epoch
            .checked_add(2)
            .ok_or(EpochActivationTargetError::ReshareTargetEpochOverflow)?;

        Ok(Self {
            boundary_epoch_e: post_advance_epoch,
            full_dkg_epoch,
            reshare_target_epoch,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorActivationSchedule {
    default_players: Vec<[u8; 32]>,
    overrides_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryValidatorActivation {
    pub targets: EpochActivationTargets,
    pub full_dkg_players: Vec<[u8; 32]>,
    pub reshare_players: Vec<[u8; 32]>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidatorActivationError {
    #[error("activation resolver missing player set for epoch {epoch}")]
    MissingPlayers { epoch: u64 },
}

impl ValidatorActivationSchedule {
    pub fn new(default_players: Vec<[u8; 32]>) -> Self {
        Self {
            default_players,
            overrides_by_epoch: BTreeMap::new(),
        }
    }

    pub fn from_parts(
        default_players: Vec<[u8; 32]>,
        overrides_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
    ) -> Self {
        Self {
            default_players,
            overrides_by_epoch,
        }
    }

    pub fn with_epoch_players(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.overrides_by_epoch.insert(epoch, players);
        self
    }

    pub fn resolve_players_for_epoch(
        &self,
        epoch: u64,
    ) -> Result<Vec<[u8; 32]>, ValidatorActivationError> {
        if self.overrides_by_epoch.is_empty() {
            return Ok(self.default_players.clone());
        }

        self.overrides_by_epoch
            .get(&epoch)
            .cloned()
            .ok_or(ValidatorActivationError::MissingPlayers { epoch })
    }

    pub fn resolve_boundary_activation(
        &self,
        targets: EpochActivationTargets,
    ) -> Result<BoundaryValidatorActivation, ValidatorActivationError> {
        Ok(BoundaryValidatorActivation {
            targets,
            full_dkg_players: self.resolve_players_for_epoch(targets.full_dkg_epoch)?,
            reshare_players: self.resolve_players_for_epoch(targets.reshare_target_epoch)?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DkgMetadataError {
    #[error(transparent)]
    ExtraData(#[from] ExtraDataError),
    #[error(transparent)]
    EpochActivationTarget(#[from] EpochActivationTargetError),
    #[error(transparent)]
    ValidatorActivation(#[from] ValidatorActivationError),
    #[error("full_dkg output.players does not match activation-resolved player set")]
    FullDkgPlayersMismatch,
    #[error("reshare section is forbidden on non-boundary blocks")]
    NonBoundaryReshare,
    #[error("full_dkg section must be present for boundary block")]
    MissingBoundaryFullDkg,
    #[error("full_dkg epoch mismatch on boundary: expected {expected}, found {found}")]
    BoundaryFullDkgEpochMismatch { expected: u64, found: u64 },
    #[error("full_dkg payload mismatch with configured candidate")]
    FullDkgPayloadMismatch,
    #[error("reshare section must be present for boundary block")]
    MissingBoundaryReshare,
    #[error("reshare target epoch mismatch on boundary: expected {expected}, found {found}")]
    BoundaryReshareTargetEpochMismatch { expected: u64, found: u64 },
    #[error("reshare players do not match activation-resolved player set")]
    ResharePlayersMismatch,
    #[error("full_dkg section must be present for this block")]
    MissingRequiredFullDkg,
    #[error("full_dkg section must be omitted for this block")]
    UnexpectedFullDkg,
    #[error("full_dkg section must be omitted when no full_dkg candidate is configured")]
    FullDkgWithoutCandidate,
    #[error("reshare section must be omitted when no full_dkg candidate is configured")]
    ReshareWithoutCandidate,
    #[error("full_dkg and reshare sections must be omitted when full_dkg feature is disabled")]
    DisabledFeatureMetadata,
    #[error("failed to decode historical block {height} extra_data: {source}")]
    HistoricalExtraDataDecode { height: u64, source: ExtraDataError },
    #[error("history storage error: {0}")]
    History(String),
}

pub struct DkgProposalInput<'a> {
    pub feature_enabled: bool,
    pub activation_schedule: &'a ValidatorActivationSchedule,
    pub default_players: &'a [[u8; 32]],
    pub previous_full_dkg: Option<&'a FullDkgV1>,
    pub candidate_output: Option<&'a FullDkgOutputV1>,
    pub proposer_public_key: [u8; 32],
    pub boundary_required: bool,
    pub post_advance_epoch: u64,
}

pub fn build_canonical_dkg_extra_data(
    input: DkgProposalInput<'_>,
) -> Result<Vec<u8>, DkgMetadataError> {
    let raw_eth = input.proposer_public_key.to_vec();

    let boundary_context = if input.boundary_required {
        Some(EpochActivationTargets::from_post_advance_epoch(
            input.post_advance_epoch,
        )?)
    } else {
        None
    };

    let candidate_epoch = boundary_context
        .map(|ctx| ctx.full_dkg_epoch)
        .unwrap_or(input.post_advance_epoch);
    let candidate_full_dkg = input
        .feature_enabled
        .then(|| input.candidate_output.cloned())
        .flatten()
        .map(|output| FullDkgV1 {
            epoch: candidate_epoch,
            output,
        });
    if let Some(candidate) = candidate_full_dkg.as_ref() {
        ensure_full_dkg_players_match_activation(input.activation_schedule, candidate)?;
    }

    let (full_dkg, reshare) = if let Some(boundary_context) = boundary_context {
        if let Some(full_dkg) = candidate_full_dkg {
            let boundary_activation = input
                .activation_schedule
                .resolve_boundary_activation(boundary_context)?;
            let reshare = ReshareV1 {
                target_epoch: boundary_context.reshare_target_epoch,
                players: boundary_activation.reshare_players,
            };
            (Some(full_dkg), Some(reshare))
        } else {
            (None, None)
        }
    } else {
        let full_dkg = candidate_full_dkg.filter(|candidate| {
            full_dkg_should_be_included(input.default_players, input.previous_full_dkg, candidate)
        });
        (full_dkg, None)
    };

    encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(raw_eth),
        full_dkg,
        reshare,
    })
    .map_err(DkgMetadataError::from)
}

pub fn full_dkg_should_be_included(
    default_players: &[[u8; 32]],
    previous_full_dkg: Option<&FullDkgV1>,
    candidate: &FullDkgV1,
) -> bool {
    let expected_players = default_players.to_vec();
    let (previous_dealers, previous_players, previous_polynomial) = match previous_full_dkg {
        Some(previous) => (
            previous.output.dealers.clone(),
            previous.output.players.clone(),
            previous.output.public_polynomial.clone(),
        ),
        None => (expected_players.clone(), expected_players, Vec::new()),
    };

    candidate.output.dealers != previous_dealers
        || candidate.output.players != previous_players
        || candidate.output.public_polynomial != previous_polynomial
}

pub fn ensure_full_dkg_players_match_activation(
    activation_schedule: &ValidatorActivationSchedule,
    full_dkg: &FullDkgV1,
) -> Result<(), DkgMetadataError> {
    let expected_players = activation_schedule.resolve_players_for_epoch(full_dkg.epoch)?;
    if full_dkg.output.players != expected_players {
        return Err(DkgMetadataError::FullDkgPlayersMismatch);
    }

    Ok(())
}

pub struct DkgVerifyInput<'a> {
    pub feature_enabled: bool,
    pub activation_schedule: &'a ValidatorActivationSchedule,
    pub default_players: &'a [[u8; 32]],
    pub previous_full_dkg: Option<&'a FullDkgV1>,
    pub candidate_output: Option<&'a FullDkgOutputV1>,
    pub boundary_required: bool,
    pub post_advance_epoch: u64,
}

pub fn validate_dkg_extra_data(
    decoded_extra_data: &CanonicalExtraDataV1,
    input: DkgVerifyInput<'_>,
) -> Result<(), DkgMetadataError> {
    let boundary_epoch_context = if input.boundary_required {
        Some(EpochActivationTargets::from_post_advance_epoch(
            input.post_advance_epoch,
        )?)
    } else {
        None
    };

    if !input.boundary_required && decoded_extra_data.reshare.is_some() {
        return Err(DkgMetadataError::NonBoundaryReshare);
    }

    let candidate_epoch = boundary_epoch_context
        .map(|ctx| ctx.full_dkg_epoch)
        .unwrap_or(input.post_advance_epoch);
    let candidate_full_dkg = input.candidate_output.cloned().map(|output| FullDkgV1 {
        epoch: candidate_epoch,
        output,
    });

    if input.feature_enabled {
        match candidate_full_dkg {
            Some(candidate_full_dkg) => {
                ensure_full_dkg_players_match_activation(
                    input.activation_schedule,
                    &candidate_full_dkg,
                )?;

                if input.boundary_required {
                    let boundary_epoch_context =
                        boundary_epoch_context.expect("context exists for boundary");
                    let observed_full_dkg = decoded_extra_data
                        .full_dkg
                        .as_ref()
                        .ok_or(DkgMetadataError::MissingBoundaryFullDkg)?;
                    if observed_full_dkg.epoch != boundary_epoch_context.full_dkg_epoch {
                        return Err(DkgMetadataError::BoundaryFullDkgEpochMismatch {
                            expected: boundary_epoch_context.full_dkg_epoch,
                            found: observed_full_dkg.epoch,
                        });
                    }
                    if observed_full_dkg != &candidate_full_dkg {
                        return Err(DkgMetadataError::FullDkgPayloadMismatch);
                    }

                    let observed_reshare = decoded_extra_data
                        .reshare
                        .as_ref()
                        .ok_or(DkgMetadataError::MissingBoundaryReshare)?;
                    if observed_reshare.target_epoch != boundary_epoch_context.reshare_target_epoch
                    {
                        return Err(DkgMetadataError::BoundaryReshareTargetEpochMismatch {
                            expected: boundary_epoch_context.reshare_target_epoch,
                            found: observed_reshare.target_epoch,
                        });
                    }
                    let expected_reshare_players = input
                        .activation_schedule
                        .resolve_players_for_epoch(boundary_epoch_context.reshare_target_epoch)?;
                    if observed_reshare.players != expected_reshare_players {
                        return Err(DkgMetadataError::ResharePlayersMismatch);
                    }
                } else {
                    let should_include = full_dkg_should_be_included(
                        input.default_players,
                        input.previous_full_dkg,
                        &candidate_full_dkg,
                    );
                    match (should_include, decoded_extra_data.full_dkg.as_ref()) {
                        (true, Some(observed)) => {
                            if observed != &candidate_full_dkg {
                                return Err(DkgMetadataError::FullDkgPayloadMismatch);
                            }
                        }
                        (true, None) => return Err(DkgMetadataError::MissingRequiredFullDkg),
                        (false, Some(_)) => return Err(DkgMetadataError::UnexpectedFullDkg),
                        (false, None) => {}
                    }
                }
            }
            None => {
                if decoded_extra_data.full_dkg.is_some() {
                    return Err(DkgMetadataError::FullDkgWithoutCandidate);
                }
                if decoded_extra_data.reshare.is_some() {
                    return Err(DkgMetadataError::ReshareWithoutCandidate);
                }
            }
        }
    } else if decoded_extra_data.full_dkg.is_some() || decoded_extra_data.reshare.is_some() {
        return Err(DkgMetadataError::DisabledFeatureMetadata);
    }

    Ok(())
}

pub trait DkgHistory {
    type Error;

    fn full_dkg_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error>;
}

pub fn latest_committed_full_dkg<History>(
    history: &History,
    start_height: u64,
) -> Result<Option<FullDkgV1>, DkgMetadataError>
where
    History: DkgHistory,
    History::Error: fmt::Display,
{
    let mut height = start_height;
    loop {
        let maybe_extra_data = history
            .full_dkg_at_height(height)
            .map_err(|err| DkgMetadataError::History(err.to_string()))?;
        if let Some(extra_data) = maybe_extra_data {
            let decoded = decode_extra_data(&extra_data)
                .map_err(|source| DkgMetadataError::HistoricalExtraDataDecode { height, source })?;
            if let Some(full_dkg) = decoded.full_dkg {
                return Ok(Some(full_dkg));
            }
        }

        if height == 0 {
            break;
        }
        height -= 1;
    }

    Ok(None)
}
