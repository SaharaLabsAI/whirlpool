use alloy_consensus::TxEip1559;
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Signature, TxKind, B256, U256};
use app::types::EvmBlock;
use reth_ethereum_primitives::{Transaction, TransactionSigned};
use rpc_eth::convert::{decode_transaction, evmblock_to_block, evmblock_to_header};
use validators_dkg::{
    encode_canonical_extra_data, CanonicalExtraDataV1, FullDkgOutputV1, FullDkgV1,
};

fn sample_signed_tx(nonce: u64, to: Address, value: u64) -> TransactionSigned {
    TransactionSigned::new_unhashed(
        Transaction::Eip1559(TxEip1559 {
            chain_id: 1,
            nonce,
            gas_limit: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            to: TxKind::Call(to),
            value: U256::from(value),
            access_list: Default::default(),
            input: Default::default(),
        }),
        Signature::test_signature(),
    )
}

fn sample_block(transactions: Vec<Vec<u8>>) -> EvmBlock {
    EvmBlock {
        height: 42,
        parent_id: [0x11; 32],
        state_root: [0x22; 32],
        transactions_root: [0x33; 32],
        receipts_root: [0x44; 32],
        proposer_public_key: [0x55; 32],
        proposer_fee_recipient: Address::repeat_byte(0x55).into_array(),
        extra_data: vec![0x55; 32],
        gas_used: 55_555,
        base_fee_per_gas: 1_000_000_000,
        timestamp: 1_700_000_123,
        transactions,
    }
}

#[test]
fn decode_transaction_roundtrips_valid_eip1559_bytes() {
    let signed = sample_signed_tx(7, Address::repeat_byte(0xaa), 42);
    let encoded = signed.encoded_2718();

    let decoded = decode_transaction(encoded.as_ref()).expect("transaction should decode");

    assert_eq!(decoded, signed);
}

#[test]
fn decode_transaction_rejects_malformed_bytes() {
    let err = decode_transaction(&[0x02]).expect_err("malformed transaction should fail");

    assert!(
        !err.to_string().is_empty(),
        "decode error should include failure details"
    );
}

#[test]
fn evmblock_to_header_maps_fields() {
    let block = sample_block(vec![]);

    let header = evmblock_to_header(&block);

    assert_eq!(header.number, block.height);
    assert_eq!(header.parent_hash, B256::from_slice(&block.parent_id));
    assert_eq!(header.state_root, B256::from_slice(&block.state_root));
    assert_eq!(
        header.transactions_root,
        B256::from_slice(&block.transactions_root)
    );
    assert_eq!(header.receipts_root, B256::from_slice(&block.receipts_root));
    assert_eq!(
        header.beneficiary,
        Address::from(block.proposer_fee_recipient)
    );
    assert_eq!(header.gas_used, block.gas_used);
    assert_eq!(header.timestamp, block.timestamp);
}

#[test]
fn evmblock_to_block_decodes_transactions() {
    let tx_one = sample_signed_tx(1, Address::repeat_byte(0x10), 100);
    let tx_two = sample_signed_tx(2, Address::repeat_byte(0x20), 200);
    let block = sample_block(vec![
        tx_one.clone().encoded_2718().to_vec(),
        tx_two.clone().encoded_2718().to_vec(),
    ]);

    let reth_block = evmblock_to_block(&block).expect("block conversion should succeed");

    assert_eq!(reth_block.header.number, block.height);
    assert_eq!(reth_block.body.transactions, vec![tx_one, tx_two]);
    assert!(reth_block.body.ommers.is_empty());
    assert!(reth_block.body.withdrawals.is_none());
}

#[test]
fn evmblock_to_block_supports_empty_transactions() {
    let block = sample_block(vec![]);

    let reth_block = evmblock_to_block(&block).expect("empty block conversion should succeed");

    assert_eq!(reth_block.header.number, block.height);
    assert!(reth_block.body.transactions.is_empty());
    assert!(reth_block.body.ommers.is_empty());
    assert!(reth_block.body.withdrawals.is_none());
}

#[test]
fn evmblock_to_header_projects_raw_eth_from_canonical_extra_data() {
    let canonical = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x99; 32]),
        full_dkg: Some(FullDkgV1 {
            epoch: 3,
            output: FullDkgOutputV1 {
                dealers: vec![[0x11; 32]],
                players: vec![[0x22; 32]],
                public_polynomial: vec![0xaa, 0xbb, 0xcc],
            },
        }),
        reshare: None,
    })
    .expect("canonical extra_data should encode");

    let mut block = sample_block(vec![]);
    block.extra_data = canonical.clone();

    let header = evmblock_to_header(&block);
    assert_eq!(header.extra_data.to_vec(), vec![0x99; 32]);
    assert_ne!(header.extra_data.to_vec(), canonical);
}
