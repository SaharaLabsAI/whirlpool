use crate::{
    compute_markdown_hash, derive_finalized_write, MemTxError, PersonalityMarkdownTx,
    SignatureScheme, MAX_PERSONALITY_MARKDOWN_BYTES,
};

fn sample_tx() -> PersonalityMarkdownTx {
    PersonalityMarkdownTx::new(
        b"signer-1".to_vec(),
        b"persona-1".to_vec(),
        7,
        b"# Persona\nBe precise.".to_vec(),
        SignatureScheme::RawSecp256k1,
        vec![0x11; 65],
    )
}

#[test]
fn rejects_oversize_markdown() {
    let mut tx = sample_tx();
    tx.markdown_bytes = vec![b'a'; MAX_PERSONALITY_MARKDOWN_BYTES + 1];
    tx.markdown_hash = compute_markdown_hash(&tx.markdown_bytes);

    let err = tx
        .validate()
        .expect_err("oversize markdown must be rejected");
    assert!(matches!(err, MemTxError::MarkdownTooLarge { .. }));
}

#[test]
fn rejects_markdown_hash_mismatch() {
    let mut tx = sample_tx();
    tx.markdown_hash = [9u8; 32];

    let err = tx
        .validate()
        .expect_err("hash mismatch must be rejected deterministically");
    assert!(matches!(err, MemTxError::MarkdownHashMismatch { .. }));
}

#[test]
fn roundtrips_canonical_payload() {
    let tx = sample_tx();
    let encoded = tx.encode().expect("encoding must succeed");
    let decoded = PersonalityMarkdownTx::decode(&encoded).expect("decode must succeed");

    assert_eq!(decoded, tx);
    assert_eq!(decoded.tx_hash().unwrap(), tx.tx_hash().unwrap());
}

#[test]
fn derives_finalized_write_from_encoded_payload() {
    let tx = sample_tx();
    let encoded = tx.encode().expect("encoding must succeed");
    let write = derive_finalized_write(&encoded).expect("write derivation must succeed");

    assert_eq!(write.personality_id, tx.personality_id);
    assert_eq!(write.markdown_bytes, tx.markdown_bytes);
    assert_eq!(write.markdown_hash, tx.markdown_hash);
    assert_eq!(write.nonce, tx.nonce);
}
