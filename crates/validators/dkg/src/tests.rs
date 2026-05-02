use std::collections::BTreeMap;

use super::{
    decide_dkg_header_sections, decode_full_dkg_v1, encode_full_dkg_v1,
    validate_dkg_header_sections, DkgHeaderDecision, DkgHeaderSectionsRef, DkgMetadataError,
    DkgProposalInput, DkgVerifyInput, FullDkgOutputV1, FullDkgV1, ReshareV1,
    ValidatorActivationSchedule,
};

fn sample_output(players: Vec<[u8; 32]>) -> FullDkgOutputV1 {
    FullDkgOutputV1 {
        dealers: vec![[0x22; 32]],
        players,
        public_polynomial: vec![0xaa, 0xbb],
    }
}

fn schedule_with_overrides(
    default_players: Vec<[u8; 32]>,
    overrides: impl IntoIterator<Item = (u64, Vec<[u8; 32]>)>,
) -> ValidatorActivationSchedule {
    ValidatorActivationSchedule::from_parts(default_players, BTreeMap::from_iter(overrides))
}

#[test]
fn full_dkg_payload_codec_roundtrips() {
    let original = FullDkgV1 {
        epoch: 7,
        output: FullDkgOutputV1 {
            dealers: vec![[0x22; 32], [0x23; 32]],
            players: vec![[0x31; 32], [0x32; 32]],
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        },
    };

    let encoded = encode_full_dkg_v1(&original).expect("encode payload");
    let decoded = decode_full_dkg_v1(&encoded).expect("decode payload");

    assert_eq!(decoded, original);
}

#[test]
fn feature_disabled_returns_empty_dkg_decision() {
    let default_players = vec![[0x11; 32]];
    let activation_schedule = ValidatorActivationSchedule::new(default_players.clone());
    let candidate_output = sample_output(default_players.clone());

    let decision = decide_dkg_header_sections(DkgProposalInput {
        feature_enabled: false,
        activation_schedule: &activation_schedule,
        default_players: &default_players,
        previous_full_dkg: None,
        candidate_output: Some(&candidate_output),
        boundary_required: false,
        post_advance_epoch: 9,
    })
    .expect("disabled feature should omit DKG sections");

    assert_eq!(decision, DkgHeaderDecision::default());
}

#[test]
fn activation_targets_are_forward_looking_from_post_advance_epoch() {
    let targets = super::EpochActivationTargets::from_post_advance_epoch(7).expect("targets");

    assert_eq!(targets.boundary_epoch_e, 7);
    assert_eq!(targets.full_dkg_epoch, 8);
    assert_eq!(targets.reshare_target_epoch, 9);
}

#[test]
fn activation_targets_fail_closed_on_overflow() {
    assert_eq!(
        super::EpochActivationTargets::from_post_advance_epoch(u64::MAX),
        Err(super::EpochActivationTargetError::FullDkgEpochOverflow)
    );
    assert_eq!(
        super::EpochActivationTargets::from_post_advance_epoch(u64::MAX - 1),
        Err(super::EpochActivationTargetError::ReshareTargetEpochOverflow)
    );
}

#[test]
fn override_schedule_is_strict_and_boundary_activation_resolves_targets() {
    let targets = super::EpochActivationTargets::from_post_advance_epoch(7).expect("targets");
    let schedule = schedule_with_overrides(
        vec![[0x11; 32]],
        [
            (targets.full_dkg_epoch, vec![[0x81; 32]]),
            (targets.reshare_target_epoch, vec![[0x91; 32]]),
        ],
    );

    assert_eq!(
        schedule.resolve_players_for_epoch(42),
        Err(super::ValidatorActivationError::MissingPlayers { epoch: 42 })
    );
    assert_eq!(
        schedule.resolve_players_for_epoch(targets.full_dkg_epoch),
        Ok(vec![[0x81; 32]])
    );
    assert_eq!(
        schedule.resolve_players_for_epoch(targets.reshare_target_epoch),
        Ok(vec![[0x91; 32]])
    );
}

#[test]
fn decision_includes_boundary_full_dkg_and_reshare() {
    let targets = super::EpochActivationTargets::from_post_advance_epoch(7).expect("targets");
    let default_players = vec![[0x11; 32]];
    let full_players = vec![[0x81; 32]];
    let reshare_players = vec![[0x91; 32]];
    let activation_schedule = schedule_with_overrides(
        default_players.clone(),
        [
            (targets.full_dkg_epoch, full_players.clone()),
            (targets.reshare_target_epoch, reshare_players.clone()),
        ],
    );
    let candidate_output = sample_output(full_players.clone());

    let decision = decide_dkg_header_sections(DkgProposalInput {
        feature_enabled: true,
        activation_schedule: &activation_schedule,
        default_players: &default_players,
        previous_full_dkg: None,
        candidate_output: Some(&candidate_output),
        boundary_required: true,
        post_advance_epoch: targets.boundary_epoch_e,
    })
    .expect("boundary decision");

    assert_eq!(
        decision.full_dkg,
        Some(FullDkgV1 {
            epoch: targets.full_dkg_epoch,
            output: candidate_output,
        })
    );
    assert_eq!(
        decision.reshare,
        Some(ReshareV1 {
            target_epoch: targets.reshare_target_epoch,
            players: reshare_players,
        })
    );
}

#[test]
fn decision_omits_unchanged_non_boundary_candidate() {
    let default_players = vec![[0x11; 32]];
    let activation_schedule = ValidatorActivationSchedule::new(default_players.clone());
    let previous = FullDkgV1 {
        epoch: 3,
        output: sample_output(default_players.clone()),
    };
    let candidate_output = previous.output.clone();

    let decision = decide_dkg_header_sections(DkgProposalInput {
        feature_enabled: true,
        activation_schedule: &activation_schedule,
        default_players: &default_players,
        previous_full_dkg: Some(&previous),
        candidate_output: Some(&candidate_output),
        boundary_required: false,
        post_advance_epoch: previous.epoch,
    })
    .expect("unchanged candidate decision");

    assert_eq!(decision, DkgHeaderDecision::default());
}

#[test]
fn full_dkg_should_be_included_when_candidate_changes() {
    let default_players = vec![[0x11; 32]];
    let activation_schedule = ValidatorActivationSchedule::new(default_players.clone());
    let previous = FullDkgV1 {
        epoch: 3,
        output: FullDkgOutputV1 {
            dealers: vec![[0x22; 32]],
            players: default_players.clone(),
            public_polynomial: vec![0xaa, 0xbb],
        },
    };
    let candidate_output = FullDkgOutputV1 {
        dealers: vec![[0x44; 32]],
        players: default_players.clone(),
        public_polynomial: vec![0xaa, 0xbb],
    };

    let decision = decide_dkg_header_sections(DkgProposalInput {
        feature_enabled: true,
        activation_schedule: &activation_schedule,
        default_players: &default_players,
        previous_full_dkg: Some(&previous),
        candidate_output: Some(&candidate_output),
        boundary_required: false,
        post_advance_epoch: 4,
    })
    .expect("changed candidate should decide");

    assert_eq!(
        decision.full_dkg,
        Some(FullDkgV1 {
            epoch: 4,
            output: candidate_output,
        })
    );
}

#[test]
fn validate_rejects_non_boundary_reshare() {
    let default_players = vec![[0x11; 32]];
    let activation_schedule = ValidatorActivationSchedule::new(default_players.clone());
    let reshare = ReshareV1 {
        target_epoch: 9,
        players: default_players.clone(),
    };

    let err = validate_dkg_header_sections(
        DkgHeaderSectionsRef {
            full_dkg: None,
            reshare: Some(&reshare),
        },
        DkgVerifyInput {
            feature_enabled: true,
            activation_schedule: &activation_schedule,
            default_players: &default_players,
            previous_full_dkg: None,
            candidate_output: None,
            boundary_required: false,
            post_advance_epoch: 7,
        },
    )
    .expect_err("non-boundary reshare must reject");

    assert!(matches!(err, DkgMetadataError::NonBoundaryReshare));
}

#[test]
fn validate_rejects_missing_boundary_sections() {
    let default_players = vec![[0x11; 32]];
    let targets = super::EpochActivationTargets::from_post_advance_epoch(7).expect("targets");
    let activation_schedule = schedule_with_overrides(
        default_players.clone(),
        [
            (targets.full_dkg_epoch, default_players.clone()),
            (targets.reshare_target_epoch, default_players.clone()),
        ],
    );
    let candidate_output = sample_output(default_players.clone());

    let err = validate_dkg_header_sections(
        DkgHeaderSectionsRef {
            full_dkg: None,
            reshare: None,
        },
        DkgVerifyInput {
            feature_enabled: true,
            activation_schedule: &activation_schedule,
            default_players: &default_players,
            previous_full_dkg: None,
            candidate_output: Some(&candidate_output),
            boundary_required: true,
            post_advance_epoch: targets.boundary_epoch_e,
        },
    )
    .expect_err("boundary DKG metadata is required");

    assert!(matches!(err, DkgMetadataError::MissingBoundaryFullDkg));
}
