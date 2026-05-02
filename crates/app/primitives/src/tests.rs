use super::{EvmBlock, ExecutionResult};
use consensus::traits::Block as CoreBlock;

fn sample_block() -> EvmBlock {
    EvmBlock {
        height: 10,
        parent_id: [1u8; 32],
        state_root: [2u8; 32],
        transactions_root: [3u8; 32],
        receipts_root: [4u8; 32],
        proposer_public_key: [5u8; 32],
        proposer_fee_recipient: [5u8; 20],
        extra_data: vec![5u8; 32],
        gas_used: 42,
        base_fee_per_gas: 1_000_000_000,
        timestamp: 1_700_000_000,
        transactions: vec![vec![0xaa, 0xbb], vec![0xcc]],
    }
}

#[test]
fn test_evm_block_trait_impl() {
    let block = sample_block();
    assert_eq!(CoreBlock::height(&block), 10);
    assert_eq!(CoreBlock::parent_id(&block), [1u8; 32]);
    assert!(CoreBlock::id(&block).iter().any(|b| *b != 0));
}

#[test]
fn test_evm_block_codec_roundtrip() {
    use commonware_codec::{Read as CodecRead, Write as CodecWrite};

    let block = sample_block();
    let mut buf = bytes::BytesMut::new();
    block.write(&mut buf);
    let decoded = EvmBlock::read_cfg(&mut buf.freeze(), &()).expect("decode should succeed");

    assert_eq!(decoded.height, block.height);
    assert_eq!(decoded.parent_id, block.parent_id);
    assert_eq!(decoded.state_root, block.state_root);
    assert_eq!(decoded.transactions_root, block.transactions_root);
    assert_eq!(decoded.receipts_root, block.receipts_root);
    assert_eq!(decoded.proposer_public_key, block.proposer_public_key);
    assert_eq!(decoded.proposer_fee_recipient, block.proposer_fee_recipient);
    assert_eq!(decoded.extra_data, block.extra_data);
    assert_eq!(decoded.gas_used, block.gas_used);
    assert_eq!(decoded.base_fee_per_gas, block.base_fee_per_gas);
    assert_eq!(decoded.timestamp, block.timestamp);
    assert_eq!(decoded.transactions, block.transactions);
}

#[test]
fn fixed_evm_block_wire_id_and_digest_fixture_is_stable() {
    use commonware_codec::Write as CodecWrite;
    use commonware_cryptography::Digestible;

    let block = sample_block();
    let mut encoded = bytes::BytesMut::new();
    block.write(&mut encoded);

    let expected_encoded = bytes_from_hex(
        "000000000000000a010101010101010101010101010101010101010101010101010101010101010102020202020202020202020202020202020202020202020202020202020202020303030303030303030303030303030303030303030303030303030303030303040404040404040404040404040404040404040404040404040404040404040405050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505000000200505050505050505050505050505050505050505050505050505050505050505000000000000002a000000003b9aca00000000006553f1000000000200000002aabb00000001cc",
    );
    assert_eq!(encoded.as_ref(), expected_encoded.as_slice());
    assert_eq!(
        block.compute_id(),
        bytes32_from_hex("7fbbcf742ba014ef47d8005d6753cd989082b17c9d0e702086d89c04abd4a200")
    );
    assert_eq!(
        block.digest().as_ref(),
        bytes32_from_hex("5696896bbec57e5d66ce8268096fe1235c95dd2c519994a132ffd960a96f49c2")
            .as_slice()
    );
}

fn bytes_from_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "hex string must contain byte pairs");
    (0..hex.len())
        .step_by(2)
        .map(|idx| u8::from_str_radix(&hex[idx..idx + 2], 16).expect("valid hex byte"))
        .collect()
}

fn bytes32_from_hex(hex: &str) -> [u8; 32] {
    let bytes = bytes_from_hex(hex);
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

#[test]
fn test_execution_result_fields() {
    let result = ExecutionResult {
        state_root: [2u8; 32],
        receipts_root: [3u8; 32],
        gas_used: 100,
        receipt_count: 5,
    };

    assert_eq!(result.state_root, [2u8; 32]);
    assert_eq!(result.receipts_root, [3u8; 32]);
    assert_eq!(result.gas_used, 100);
    assert_eq!(result.receipt_count, 5);
}

#[test]
fn canonical_header_extra_data_roundtrips_and_projects() {
    use super::header_extra_data::{
        project_raw_eth_extra_data, CanonicalHeaderExtraDataV1, DkgHeaderSections,
    };
    use validators_dkg::{FullDkgOutputV1, FullDkgV1, ReshareV1};

    let original = CanonicalHeaderExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        dkg: DkgHeaderSections {
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
        },
    };

    let encoded =
        super::header_extra_data::encode_header_extra_data(&original).expect("encode header");
    let decoded =
        super::header_extra_data::decode_header_extra_data(&encoded).expect("decode header");

    assert_eq!(decoded, original);
    assert_eq!(project_raw_eth_extra_data(&encoded), vec![0x11; 32]);
}

#[test]
fn strict_header_extra_data_rejects_legacy_proposer_bytes() {
    let legacy = vec![0x11; 32];

    assert!(matches!(
        super::header_extra_data::decode_header_extra_data(&legacy),
        Err(super::header_extra_data::HeaderExtraDataError::InvalidMagic)
    ));
    assert_eq!(
        super::header_extra_data::project_raw_eth_extra_data(&legacy),
        Vec::<u8>::new(),
        "projection must not treat legacy proposer bytes as canonical input"
    );
}

#[test]
fn canonical_raw_eth_only_header_extra_data_roundtrips_and_projects() {
    use super::header_extra_data::{
        decode_header_extra_data, project_raw_eth_extra_data, CanonicalHeaderExtraDataV1,
        DkgHeaderSections,
    };

    let raw_eth = vec![0x42; 32];
    let canonical = CanonicalHeaderExtraDataV1 {
        raw_eth: Some(raw_eth.clone()),
        dkg: DkgHeaderSections::default(),
    };

    let encoded = super::header_extra_data::encode_header_extra_data(&canonical)
        .expect("encode raw_eth-only envelope");
    assert_ne!(
        encoded, raw_eth,
        "canonical carrier must not be legacy proposer bytes"
    );
    assert_eq!(
        decode_header_extra_data(&encoded).expect("decode"),
        canonical
    );
    assert_eq!(project_raw_eth_extra_data(&encoded), raw_eth);
}

#[test]
fn unknown_header_extra_data_section_is_rejected() {
    let mut encoded = vec![];
    encoded.extend_from_slice(b"WDX1");
    encoded.push(1); // version
    encoded.push(1); // section count
    encoded.push(9); // unknown section id
    encoded.extend_from_slice(&1u32.to_le_bytes());
    encoded.push(0xaa);

    assert!(matches!(
        super::header_extra_data::decode_header_extra_data(&encoded),
        Err(super::header_extra_data::HeaderExtraDataError::UnknownSection { section: 9 })
    ));
}

#[test]
fn section_order_rejected_when_raw_eth_after_full_dkg() {
    use super::header_extra_data::{CanonicalHeaderExtraDataV1, DkgHeaderSections};
    use validators_dkg::{FullDkgOutputV1, FullDkgV1};

    let canonical =
        super::header_extra_data::encode_header_extra_data(&CanonicalHeaderExtraDataV1 {
            raw_eth: Some(vec![0x11; 32]),
            dkg: DkgHeaderSections {
                full_dkg: Some(FullDkgV1 {
                    epoch: 2,
                    output: FullDkgOutputV1 {
                        dealers: vec![[0x21; 32]],
                        players: vec![[0x31; 32]],
                        public_polynomial: vec![0xaa, 0xbb],
                    },
                }),
                reshare: None,
            },
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

    let err = super::header_extra_data::decode_header_extra_data(&reordered)
        .expect_err("raw_eth after full_dkg must be rejected");
    assert!(matches!(
        err,
        super::header_extra_data::HeaderExtraDataError::InvalidSectionOrder { section } if section == 1
    ));
}

#[test]
fn section_order_rejected_when_reshare_before_full_dkg() {
    use super::header_extra_data::{CanonicalHeaderExtraDataV1, DkgHeaderSections};
    use validators_dkg::{FullDkgOutputV1, FullDkgV1, ReshareV1};

    let canonical =
        super::header_extra_data::encode_header_extra_data(&CanonicalHeaderExtraDataV1 {
            raw_eth: Some(vec![0x11; 32]),
            dkg: DkgHeaderSections {
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
            },
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

    let err = super::header_extra_data::decode_header_extra_data(&reordered)
        .expect_err("reshare before full_dkg must be rejected");
    assert!(matches!(
        err,
        super::header_extra_data::HeaderExtraDataError::InvalidSectionOrder { section } if section == 3
    ));
}

#[test]
fn fixed_wdx1_full_dkg_and_reshare_fixture_is_stable() {
    use super::header_extra_data::{CanonicalHeaderExtraDataV1, DkgHeaderSections};
    use validators_dkg::{FullDkgOutputV1, FullDkgV1, ReshareV1};

    let expected = bytes_from_hex(
        "57445831010301200000001111111111111111111111111111111111111111111111111111111111111111029700000007000000000000000200000022222222222222222222222222222222222222222222222222222222222222222323232323232323232323232323232323232323232323232323232323232323020000003131313131313131313131313131313131313131313131313131313131313131323232323232323232323232323232323232323232323232323232323232323203000000aabbcc034c00000009000000000000000200000041414141414141414141414141414141414141414141414141414141414141414242424242424242424242424242424242424242424242424242424242424242",
    );
    let fixture = CanonicalHeaderExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        dkg: DkgHeaderSections {
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
        },
    };

    assert_eq!(
        super::header_extra_data::encode_header_extra_data(&fixture).expect("encode fixture"),
        expected
    );
    assert_eq!(
        super::header_extra_data::decode_header_extra_data(&expected).expect("decode fixture"),
        fixture
    );
}
