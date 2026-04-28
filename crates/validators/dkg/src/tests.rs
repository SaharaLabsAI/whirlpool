use super::{
    decode_extra_data, encode_canonical_extra_data, project_raw_eth_extra_data,
    CanonicalExtraDataV1, ExtraDataDecodeMode, ExtraDataError, FullDkgOutputV1, FullDkgV1,
    ReshareV1,
};

#[test]
fn test_canonical_extra_data_roundtrip_with_raw_eth_and_full_dkg() {
    let original = CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(FullDkgV1 {
            epoch: 7,
            output: FullDkgOutputV1 {
                dealers: vec![[0x22; 32], [0x23; 32]],
                players: vec![[0x31; 32], [0x32; 32]],
                public_polynomial: vec![0xaa, 0xbb, 0xcc],
            },
        }),
        reshare: Some(ReshareV1 {
            target_epoch: 9,
            players: vec![[0x41; 32], [0x42; 32]],
        }),
    };

    let encoded = encode_canonical_extra_data(&original).expect("encode");
    let decoded = decode_extra_data(&encoded, ExtraDataDecodeMode::Strict).expect("decode");
    assert_eq!(decoded, original);

    let projected = project_raw_eth_extra_data(&encoded);
    assert_eq!(projected, vec![0x11; 32]);
}

#[test]
fn test_legacy_extra_data_decode_and_projection() {
    let legacy = vec![0x55; 32];
    let decoded = decode_extra_data(&legacy, ExtraDataDecodeMode::Legacy).expect("legacy");
    assert_eq!(decoded.raw_eth, Some(legacy.clone()));
    assert_eq!(decoded.full_dkg, None);
    assert_eq!(project_raw_eth_extra_data(&legacy), legacy);
}

#[test]
fn test_strict_mode_rejects_legacy_bytes() {
    let legacy = vec![0x11; 32];
    assert!(decode_extra_data(&legacy, ExtraDataDecodeMode::Strict).is_err());
}

#[test]
fn test_unknown_section_rejected() {
    let mut encoded = vec![];
    encoded.extend_from_slice(b"WDX1");
    encoded.push(1); // version
    encoded.push(1); // section count
    encoded.push(9); // unknown section id
    encoded.extend_from_slice(&1u32.to_le_bytes());
    encoded.push(0xaa);

    assert!(decode_extra_data(&encoded, ExtraDataDecodeMode::Legacy).is_err());
}

#[test]
fn test_section_order_rejected_when_raw_eth_after_full_dkg() {
    let canonical = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(FullDkgV1 {
            epoch: 2,
            output: FullDkgOutputV1 {
                dealers: vec![[0x21; 32]],
                players: vec![[0x31; 32]],
                public_polynomial: vec![0xaa, 0xbb],
            },
        }),
        reshare: None,
    })
    .expect("canonical envelope should encode");

    let mut cursor = &canonical[6..];
    let section1_id = cursor[0];
    let section1_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
    let section1_payload = cursor[5..5 + section1_len].to_vec();
    cursor = &cursor[5 + section1_len..];

    let section2_id = cursor[0];
    let section2_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
    let section2_payload = cursor[5..5 + section2_len].to_vec();
    assert_eq!(section1_id, 1, "first canonical section should be raw_eth");
    assert_eq!(
        section2_id, 2,
        "second canonical section should be full_dkg"
    );

    let mut reordered = Vec::new();
    reordered.extend_from_slice(b"WDX1");
    reordered.push(1);
    reordered.push(2);

    reordered.push(section2_id);
    reordered.extend_from_slice(&(section2_payload.len() as u32).to_le_bytes());
    reordered.extend_from_slice(&section2_payload);

    reordered.push(section1_id);
    reordered.extend_from_slice(&(section1_payload.len() as u32).to_le_bytes());
    reordered.extend_from_slice(&section1_payload);

    let err = decode_extra_data(&reordered, ExtraDataDecodeMode::Strict)
        .expect_err("raw_eth after full_dkg must be rejected");
    assert!(matches!(
        err,
        ExtraDataError::InvalidSectionOrder { section } if section == 1
    ));
}

#[test]
fn test_section_order_rejected_when_reshare_before_full_dkg() {
    let canonical = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(FullDkgV1 {
            epoch: 2,
            output: FullDkgOutputV1 {
                dealers: vec![[0x21; 32]],
                players: vec![[0x31; 32]],
                public_polynomial: vec![0xaa, 0xbb],
            },
        }),
        reshare: Some(ReshareV1 {
            target_epoch: 3,
            players: vec![[0x41; 32]],
        }),
    })
    .expect("canonical envelope should encode");

    let mut cursor = &canonical[6..];
    let raw_id = cursor[0];
    let raw_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
    let raw_payload = cursor[5..5 + raw_len].to_vec();
    cursor = &cursor[5 + raw_len..];

    let full_dkg_id = cursor[0];
    let full_dkg_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
    let full_dkg_payload = cursor[5..5 + full_dkg_len].to_vec();
    cursor = &cursor[5 + full_dkg_len..];

    let reshare_id = cursor[0];
    let reshare_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
    let reshare_payload = cursor[5..5 + reshare_len].to_vec();

    let mut reordered = Vec::new();
    reordered.extend_from_slice(b"WDX1");
    reordered.push(1);
    reordered.push(3);

    reordered.push(raw_id);
    reordered.extend_from_slice(&(raw_payload.len() as u32).to_le_bytes());
    reordered.extend_from_slice(&raw_payload);

    reordered.push(reshare_id);
    reordered.extend_from_slice(&(reshare_payload.len() as u32).to_le_bytes());
    reordered.extend_from_slice(&reshare_payload);

    reordered.push(full_dkg_id);
    reordered.extend_from_slice(&(full_dkg_payload.len() as u32).to_le_bytes());
    reordered.extend_from_slice(&full_dkg_payload);

    let err = decode_extra_data(&reordered, ExtraDataDecodeMode::Strict)
        .expect_err("reshare before full_dkg must be rejected");
    assert!(matches!(
        err,
        ExtraDataError::InvalidSectionOrder { section } if section == 3
    ));
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
    let schedule = super::ValidatorActivationSchedule::new(vec![[0x11; 32]])
        .with_epoch_players(targets.full_dkg_epoch, vec![[0x81; 32]])
        .with_epoch_players(targets.reshare_target_epoch, vec![[0x91; 32]]);

    assert_eq!(
        schedule.resolve_players_for_epoch(42),
        Err(super::ValidatorActivationError::MissingPlayers { epoch: 42 })
    );
    let activation = schedule
        .resolve_boundary_activation(targets)
        .expect("boundary activation");

    assert_eq!(activation.targets, targets);
    assert_eq!(activation.full_dkg_players, vec![[0x81; 32]]);
    assert_eq!(activation.reshare_players, vec![[0x91; 32]]);
}

#[derive(Default)]
struct TestHistory {
    blocks: std::collections::BTreeMap<u64, Vec<u8>>,
}

impl super::DkgExtraDataHistory for TestHistory {
    type Error = String;

    fn extra_data_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.blocks.get(&height).cloned())
    }
}

#[test]
fn latest_committed_full_dkg_scans_past_raw_only_intermediate_blocks() {
    let full_dkg = FullDkgV1 {
        epoch: 3,
        output: FullDkgOutputV1 {
            dealers: vec![[0x11; 32]],
            players: vec![[0x21; 32]],
            public_polynomial: vec![0xaa],
        },
    };
    let full_dkg_extra = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x33; 32]),
        full_dkg: Some(full_dkg.clone()),
        reshare: None,
    })
    .expect("full dkg extra_data");
    let raw_extra = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x44; 32]),
        full_dkg: None,
        reshare: None,
    })
    .expect("raw extra_data");
    let mut history = TestHistory::default();
    history.blocks.insert(0, full_dkg_extra);
    history.blocks.insert(1, raw_extra.clone());
    history.blocks.insert(2, raw_extra);

    assert_eq!(
        super::latest_committed_full_dkg(&history, 2).expect("scan"),
        Some(full_dkg)
    );
}

fn bytes_from_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "hex string must contain byte pairs");
    (0..hex.len())
        .step_by(2)
        .map(|idx| u8::from_str_radix(&hex[idx..idx + 2], 16).expect("valid hex byte"))
        .collect()
}

#[test]
fn fixed_legacy_raw_extra_data_fixture_is_stable() {
    let expected =
        bytes_from_hex("5555555555555555555555555555555555555555555555555555555555555555");
    let decoded = decode_extra_data(&expected, ExtraDataDecodeMode::Legacy).expect("legacy decode");

    assert_eq!(decoded.raw_eth, Some(expected.clone()));
    assert_eq!(
        super::legacy_proposer_extra_data_bytes([0x55; 32]),
        expected
    );
    assert!(matches!(
        decode_extra_data(&expected, ExtraDataDecodeMode::Strict),
        Err(ExtraDataError::InvalidMagic)
    ));
}

#[test]
fn fixed_wdx1_full_dkg_and_reshare_fixture_is_stable() {
    let expected = bytes_from_hex(
        "57445831010301200000001111111111111111111111111111111111111111111111111111111111111111029700000007000000000000000200000022222222222222222222222222222222222222222222222222222222222222222323232323232323232323232323232323232323232323232323232323232323020000003131313131313131313131313131313131313131313131313131313131313131323232323232323232323232323232323232323232323232323232323232323203000000aabbcc034c00000009000000000000000200000041414141414141414141414141414141414141414141414141414141414141414242424242424242424242424242424242424242424242424242424242424242",
    );
    let fixture = CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(FullDkgV1 {
            epoch: 7,
            output: FullDkgOutputV1 {
                dealers: vec![[0x22; 32], [0x23; 32]],
                players: vec![[0x31; 32], [0x32; 32]],
                public_polynomial: vec![0xaa, 0xbb, 0xcc],
            },
        }),
        reshare: Some(ReshareV1 {
            target_epoch: 9,
            players: vec![[0x41; 32], [0x42; 32]],
        }),
    };

    assert_eq!(
        encode_canonical_extra_data(&fixture).expect("encode fixture"),
        expected
    );
    assert_eq!(
        decode_extra_data(&expected, ExtraDataDecodeMode::Strict).expect("decode fixture"),
        fixture
    );
}
