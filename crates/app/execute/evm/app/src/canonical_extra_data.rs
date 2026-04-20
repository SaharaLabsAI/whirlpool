use app::{
    encode_canonical_extra_data, legacy_proposer_extra_data_bytes, CanonicalExtraDataV1, FullDkgV1,
    ReshareV1,
};

use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;
use crate::validator_activation::{ActivationSourceResolver, BoundaryEpochContext};

pub fn full_dkg_should_be_included(
    evm_config: &WhirlpoolEvmConfig,
    previous_full_dkg: Option<&FullDkgV1>,
    candidate: &FullDkgV1,
) -> bool {
    let expected_players = evm_config.simplex_consensus_public_keys();
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
    activation_resolver: &ActivationSourceResolver<'_>,
    full_dkg: &FullDkgV1,
) -> Result<(), EvmAppError> {
    let expected_players = activation_resolver.resolve_players_for_epoch(full_dkg.epoch)?;
    if full_dkg.output.players != expected_players {
        return Err(EvmAppError::InvalidBlock(
            "full_dkg output.players does not match activation-resolved player set".into(),
        ));
    }

    Ok(())
}

pub fn build_canonical_extra_data(
    evm_config: &WhirlpoolEvmConfig,
    previous_full_dkg: Option<&FullDkgV1>,
    proposer_public_key: [u8; 32],
    boundary_required: bool,
    epoch: u64,
) -> Result<Vec<u8>, EvmAppError> {
    let raw_eth = legacy_proposer_extra_data_bytes(proposer_public_key);

    if !evm_config.full_dkg_feature_enabled() {
        return Ok(raw_eth);
    }

    let activation_resolver = ActivationSourceResolver::new(evm_config);
    let boundary_context = if boundary_required {
        Some(BoundaryEpochContext::from_post_advance_epoch(epoch)?)
    } else {
        None
    };
    let candidate_epoch = boundary_context
        .map(|ctx| ctx.full_dkg_epoch)
        .unwrap_or(epoch);

    let candidate_full_dkg = evm_config.current_full_dkg_payload(candidate_epoch);
    if let Some(candidate) = candidate_full_dkg.as_ref() {
        ensure_full_dkg_players_match_activation(&activation_resolver, candidate)?;
    }

    let (full_dkg, reshare) = if let Some(boundary_context) = boundary_context {
        if let Some(full_dkg) = candidate_full_dkg {
            let reshare_players = activation_resolver
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
            full_dkg_should_be_included(evm_config, previous_full_dkg, candidate)
        });
        (full_dkg, None)
    };

    encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(raw_eth),
        full_dkg,
        reshare,
    })
    .map_err(|err| EvmAppError::InvalidBlock(format!("invalid canonical extra_data: {err}")))
}
