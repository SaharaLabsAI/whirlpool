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
