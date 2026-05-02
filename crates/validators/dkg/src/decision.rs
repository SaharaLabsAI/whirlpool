use crate::{
    DkgHeaderDecision, DkgHeaderSectionsRef, DkgPayloadError, EpochActivationTargetError,
    EpochActivationTargets, FullDkgOutputV1, FullDkgV1, ReshareV1, ValidatorActivationError,
    ValidatorActivationSchedule,
};

#[derive(Debug, thiserror::Error)]
pub enum DkgMetadataError {
    #[error(transparent)]
    Payload(#[from] DkgPayloadError),
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
}

pub struct DkgProposalInput<'a> {
    pub feature_enabled: bool,
    pub activation_schedule: &'a ValidatorActivationSchedule,
    pub default_players: &'a [[u8; 32]],
    pub previous_full_dkg: Option<&'a FullDkgV1>,
    pub candidate_output: Option<&'a FullDkgOutputV1>,
    pub boundary_required: bool,
    pub post_advance_epoch: u64,
}

pub fn decide_dkg_header_sections(
    input: DkgProposalInput<'_>,
) -> Result<DkgHeaderDecision, DkgMetadataError> {
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
            let reshare_players = input
                .activation_schedule
                .resolve_players_for_epoch(boundary_context.reshare_target_epoch)?;
            let reshare = ReshareV1 {
                target_epoch: boundary_context.reshare_target_epoch,
                players: reshare_players,
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

    Ok(DkgHeaderDecision { full_dkg, reshare })
}

fn full_dkg_should_be_included(
    default_players: &[[u8; 32]],
    previous_full_dkg: Option<&FullDkgV1>,
    candidate: &FullDkgV1,
) -> bool {
    let (previous_dealers, previous_players, previous_polynomial): (
        &[[u8; 32]],
        &[[u8; 32]],
        &[u8],
    ) = match previous_full_dkg {
        Some(previous) => (
            previous.output.dealers.as_slice(),
            previous.output.players.as_slice(),
            previous.output.public_polynomial.as_slice(),
        ),
        None => (default_players, default_players, &[]),
    };

    candidate.output.dealers.as_slice() != previous_dealers
        || candidate.output.players.as_slice() != previous_players
        || candidate.output.public_polynomial.as_slice() != previous_polynomial
}

fn ensure_full_dkg_players_match_activation(
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

pub fn validate_dkg_header_sections(
    sections: DkgHeaderSectionsRef<'_>,
    input: DkgVerifyInput<'_>,
) -> Result<(), DkgMetadataError> {
    if !input.boundary_required && sections.reshare.is_some() {
        return Err(DkgMetadataError::NonBoundaryReshare);
    }

    if !input.feature_enabled {
        return validate_disabled_feature_sections(sections);
    }

    let boundary_epoch_context = if input.boundary_required {
        Some(EpochActivationTargets::from_post_advance_epoch(
            input.post_advance_epoch,
        )?)
    } else {
        None
    };
    let candidate_epoch = boundary_epoch_context
        .map(|ctx| ctx.full_dkg_epoch)
        .unwrap_or(input.post_advance_epoch);

    let Some(candidate_output) = input.candidate_output else {
        return validate_no_candidate_sections(sections);
    };
    let candidate_full_dkg = FullDkgV1 {
        epoch: candidate_epoch,
        output: candidate_output.clone(),
    };

    ensure_full_dkg_players_match_activation(input.activation_schedule, &candidate_full_dkg)?;

    if let Some(boundary_epoch_context) = boundary_epoch_context {
        validate_boundary_sections(
            sections,
            input.activation_schedule,
            &candidate_full_dkg,
            boundary_epoch_context,
        )
    } else {
        validate_non_boundary_sections(
            sections,
            input.default_players,
            input.previous_full_dkg,
            &candidate_full_dkg,
        )
    }
}

fn validate_disabled_feature_sections(
    sections: DkgHeaderSectionsRef<'_>,
) -> Result<(), DkgMetadataError> {
    if sections.full_dkg.is_some() || sections.reshare.is_some() {
        return Err(DkgMetadataError::DisabledFeatureMetadata);
    }

    Ok(())
}

fn validate_no_candidate_sections(
    sections: DkgHeaderSectionsRef<'_>,
) -> Result<(), DkgMetadataError> {
    if sections.full_dkg.is_some() {
        return Err(DkgMetadataError::FullDkgWithoutCandidate);
    }
    if sections.reshare.is_some() {
        return Err(DkgMetadataError::ReshareWithoutCandidate);
    }

    Ok(())
}

fn validate_boundary_sections(
    sections: DkgHeaderSectionsRef<'_>,
    activation_schedule: &ValidatorActivationSchedule,
    candidate_full_dkg: &FullDkgV1,
    boundary_epoch_context: EpochActivationTargets,
) -> Result<(), DkgMetadataError> {
    let observed_full_dkg = sections
        .full_dkg
        .ok_or(DkgMetadataError::MissingBoundaryFullDkg)?;
    if observed_full_dkg.epoch != boundary_epoch_context.full_dkg_epoch {
        return Err(DkgMetadataError::BoundaryFullDkgEpochMismatch {
            expected: boundary_epoch_context.full_dkg_epoch,
            found: observed_full_dkg.epoch,
        });
    }
    if observed_full_dkg != candidate_full_dkg {
        return Err(DkgMetadataError::FullDkgPayloadMismatch);
    }

    let observed_reshare = sections
        .reshare
        .ok_or(DkgMetadataError::MissingBoundaryReshare)?;
    if observed_reshare.target_epoch != boundary_epoch_context.reshare_target_epoch {
        return Err(DkgMetadataError::BoundaryReshareTargetEpochMismatch {
            expected: boundary_epoch_context.reshare_target_epoch,
            found: observed_reshare.target_epoch,
        });
    }
    let expected_reshare_players = activation_schedule
        .resolve_players_for_epoch(boundary_epoch_context.reshare_target_epoch)?;
    if observed_reshare.players != expected_reshare_players {
        return Err(DkgMetadataError::ResharePlayersMismatch);
    }

    Ok(())
}

fn validate_non_boundary_sections(
    sections: DkgHeaderSectionsRef<'_>,
    default_players: &[[u8; 32]],
    previous_full_dkg: Option<&FullDkgV1>,
    candidate_full_dkg: &FullDkgV1,
) -> Result<(), DkgMetadataError> {
    let should_include =
        full_dkg_should_be_included(default_players, previous_full_dkg, candidate_full_dkg);
    match (should_include, sections.full_dkg) {
        (true, Some(observed)) if observed != candidate_full_dkg => {
            Err(DkgMetadataError::FullDkgPayloadMismatch)
        }
        (true, Some(_)) => Ok(()),
        (true, None) => Err(DkgMetadataError::MissingRequiredFullDkg),
        (false, Some(_)) => Err(DkgMetadataError::UnexpectedFullDkg),
        (false, None) => Ok(()),
    }
}
