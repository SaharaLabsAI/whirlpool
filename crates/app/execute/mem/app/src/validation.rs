use crate::{
    hash::compute_markdown_hash, MemTxError, PersonalityMarkdownTx, MAX_IDENTITY_BYTES,
    MAX_PERSONALITY_MARKDOWN_BYTES, MAX_SIGNATURE_BYTES, SUPPORTED_PERSONALITY_TX_VERSION,
};

pub fn validate_personality_tx(tx: &PersonalityMarkdownTx) -> Result<(), MemTxError> {
    if tx.version != SUPPORTED_PERSONALITY_TX_VERSION {
        return Err(MemTxError::UnsupportedVersion(tx.version));
    }
    if tx.signer.is_empty() {
        return Err(MemTxError::EmptySigner);
    }
    if tx.signer.len() > MAX_IDENTITY_BYTES {
        return Err(MemTxError::SignerTooLarge {
            len: tx.signer.len(),
            max: MAX_IDENTITY_BYTES,
        });
    }
    if tx.personality_id.is_empty() {
        return Err(MemTxError::EmptyPersonalityId);
    }
    if tx.personality_id.len() > MAX_IDENTITY_BYTES {
        return Err(MemTxError::PersonalityIdTooLarge {
            len: tx.personality_id.len(),
            max: MAX_IDENTITY_BYTES,
        });
    }
    if tx.markdown_bytes.is_empty() {
        return Err(MemTxError::EmptyMarkdown);
    }
    if tx.markdown_bytes.len() > MAX_PERSONALITY_MARKDOWN_BYTES {
        return Err(MemTxError::MarkdownTooLarge {
            len: tx.markdown_bytes.len(),
            max: MAX_PERSONALITY_MARKDOWN_BYTES,
        });
    }
    if std::str::from_utf8(&tx.markdown_bytes).is_err() {
        return Err(MemTxError::InvalidUtf8Markdown);
    }
    let computed = compute_markdown_hash(&tx.markdown_bytes);
    if tx.markdown_hash != computed {
        return Err(MemTxError::MarkdownHashMismatch {
            expected: tx.markdown_hash,
            computed,
        });
    }
    if tx.signature.is_empty() {
        return Err(MemTxError::EmptySignature);
    }
    if tx.signature.len() > MAX_SIGNATURE_BYTES {
        return Err(MemTxError::SignatureTooLarge {
            len: tx.signature.len(),
            max: MAX_SIGNATURE_BYTES,
        });
    }
    Ok(())
}
