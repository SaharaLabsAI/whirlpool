mod error;

pub use error::MemTxError;

use sha2::{Digest, Sha256};

pub const PERSONALITY_MARKDOWN_TYPE_ID: u32 = 1;
pub const SUPPORTED_PERSONALITY_TX_VERSION: u8 = 1;
pub const MAX_PERSONALITY_MARKDOWN_BYTES: usize = 16 * 1024;
pub const MAX_IDENTITY_BYTES: usize = 256;
pub const MAX_SIGNATURE_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SignatureScheme {
    RawSecp256k1 = 1,
}

impl SignatureScheme {
    pub fn from_wire(value: u8) -> Result<Self, MemTxError> {
        match value {
            1 => Ok(Self::RawSecp256k1),
            other => Err(MemTxError::UnsupportedSignatureScheme(other)),
        }
    }

    pub fn to_wire(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalityMarkdownTx {
    pub version: u8,
    pub signer: Vec<u8>,
    pub personality_id: Vec<u8>,
    pub nonce: u64,
    pub markdown_bytes: Vec<u8>,
    pub markdown_hash: [u8; 32],
    pub signature_scheme: SignatureScheme,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalizedPersonalityWrite {
    pub signer: Vec<u8>,
    pub personality_id: Vec<u8>,
    pub nonce: u64,
    pub markdown_bytes: Vec<u8>,
    pub markdown_hash: [u8; 32],
}

impl PersonalityMarkdownTx {
    pub fn new(
        signer: Vec<u8>,
        personality_id: Vec<u8>,
        nonce: u64,
        markdown_bytes: Vec<u8>,
        signature_scheme: SignatureScheme,
        signature: Vec<u8>,
    ) -> Self {
        let markdown_hash = compute_markdown_hash(&markdown_bytes);
        Self {
            version: SUPPORTED_PERSONALITY_TX_VERSION,
            signer,
            personality_id,
            nonce,
            markdown_bytes,
            markdown_hash,
            signature_scheme,
            signature,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, MemTxError> {
        self.validate()?;

        let mut encoded = Vec::with_capacity(encoded_len(self));
        encoded.push(self.version);
        put_len_prefixed(&mut encoded, &self.signer)?;
        put_len_prefixed(&mut encoded, &self.personality_id)?;
        encoded.extend_from_slice(&self.nonce.to_le_bytes());
        put_len_prefixed(&mut encoded, &self.markdown_bytes)?;
        encoded.extend_from_slice(&self.markdown_hash);
        encoded.push(self.signature_scheme.to_wire());
        put_len_prefixed(&mut encoded, &self.signature)?;
        Ok(encoded)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MemTxError> {
        let mut cursor = 0usize;
        let version = read_u8(bytes, &mut cursor)?;
        let signer = read_len_prefixed(bytes, &mut cursor)?;
        let personality_id = read_len_prefixed(bytes, &mut cursor)?;
        let nonce = read_u64(bytes, &mut cursor)?;
        let markdown_bytes = read_len_prefixed(bytes, &mut cursor)?;
        let markdown_hash = read_fixed_32(bytes, &mut cursor)?;
        let signature_scheme = SignatureScheme::from_wire(read_u8(bytes, &mut cursor)?)?;
        let signature = read_len_prefixed(bytes, &mut cursor)?;

        if cursor != bytes.len() {
            return Err(MemTxError::TrailingBytes {
                trailing: bytes.len() - cursor,
            });
        }

        let tx = Self {
            version,
            signer,
            personality_id,
            nonce,
            markdown_bytes,
            markdown_hash,
            signature_scheme,
            signature,
        };
        tx.validate()?;
        Ok(tx)
    }

    pub fn validate(&self) -> Result<(), MemTxError> {
        if self.version != SUPPORTED_PERSONALITY_TX_VERSION {
            return Err(MemTxError::UnsupportedVersion(self.version));
        }
        if self.signer.is_empty() {
            return Err(MemTxError::EmptySigner);
        }
        if self.signer.len() > MAX_IDENTITY_BYTES {
            return Err(MemTxError::SignerTooLarge {
                len: self.signer.len(),
                max: MAX_IDENTITY_BYTES,
            });
        }
        if self.personality_id.is_empty() {
            return Err(MemTxError::EmptyPersonalityId);
        }
        if self.personality_id.len() > MAX_IDENTITY_BYTES {
            return Err(MemTxError::PersonalityIdTooLarge {
                len: self.personality_id.len(),
                max: MAX_IDENTITY_BYTES,
            });
        }
        if self.markdown_bytes.is_empty() {
            return Err(MemTxError::EmptyMarkdown);
        }
        if self.markdown_bytes.len() > MAX_PERSONALITY_MARKDOWN_BYTES {
            return Err(MemTxError::MarkdownTooLarge {
                len: self.markdown_bytes.len(),
                max: MAX_PERSONALITY_MARKDOWN_BYTES,
            });
        }
        if std::str::from_utf8(&self.markdown_bytes).is_err() {
            return Err(MemTxError::InvalidUtf8Markdown);
        }
        let computed = compute_markdown_hash(&self.markdown_bytes);
        if self.markdown_hash != computed {
            return Err(MemTxError::MarkdownHashMismatch {
                expected: self.markdown_hash,
                computed,
            });
        }
        if self.signature.is_empty() {
            return Err(MemTxError::EmptySignature);
        }
        if self.signature.len() > MAX_SIGNATURE_BYTES {
            return Err(MemTxError::SignatureTooLarge {
                len: self.signature.len(),
                max: MAX_SIGNATURE_BYTES,
            });
        }
        Ok(())
    }

    pub fn finalized_write(&self) -> Result<FinalizedPersonalityWrite, MemTxError> {
        self.validate()?;
        Ok(FinalizedPersonalityWrite {
            signer: self.signer.clone(),
            personality_id: self.personality_id.clone(),
            nonce: self.nonce,
            markdown_bytes: self.markdown_bytes.clone(),
            markdown_hash: self.markdown_hash,
        })
    }

    pub fn tx_hash(&self) -> Result<[u8; 32], MemTxError> {
        let encoded = self.encode()?;
        Ok(compute_payload_hash(&encoded))
    }
}

pub fn decode_personality_tx(bytes: &[u8]) -> Result<PersonalityMarkdownTx, MemTxError> {
    PersonalityMarkdownTx::decode(bytes)
}

pub fn derive_finalized_write(bytes: &[u8]) -> Result<FinalizedPersonalityWrite, MemTxError> {
    PersonalityMarkdownTx::decode(bytes)?.finalized_write()
}

pub fn compute_markdown_hash(markdown_bytes: &[u8]) -> [u8; 32] {
    compute_payload_hash(markdown_bytes)
}

fn compute_payload_hash(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn encoded_len(tx: &PersonalityMarkdownTx) -> usize {
    1 + 4
        + tx.signer.len()
        + 4
        + tx.personality_id.len()
        + 8
        + 4
        + tx.markdown_bytes.len()
        + 32
        + 1
        + 4
        + tx.signature.len()
}

fn put_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) -> Result<(), MemTxError> {
    let len = u32::try_from(bytes.len()).map_err(|_| MemTxError::LengthOverflow(bytes.len()))?;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    Ok(())
}

fn read_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, MemTxError> {
    if *cursor >= bytes.len() {
        return Err(MemTxError::UnexpectedEof);
    }
    let value = bytes[*cursor];
    *cursor += 1;
    Ok(value)
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, MemTxError> {
    let slice = read_exact(bytes, cursor, 8)?;
    let mut array = [0u8; 8];
    array.copy_from_slice(slice);
    Ok(u64::from_le_bytes(array))
}

fn read_fixed_32(bytes: &[u8], cursor: &mut usize) -> Result<[u8; 32], MemTxError> {
    let slice = read_exact(bytes, cursor, 32)?;
    let mut array = [0u8; 32];
    array.copy_from_slice(slice);
    Ok(array)
}

fn read_len_prefixed(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>, MemTxError> {
    let len_bytes = read_exact(bytes, cursor, 4)?;
    let mut len_array = [0u8; 4];
    len_array.copy_from_slice(len_bytes);
    let len = u32::from_le_bytes(len_array) as usize;
    Ok(read_exact(bytes, cursor, len)?.to_vec())
}

fn read_exact<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], MemTxError> {
    let end = cursor
        .checked_add(len)
        .ok_or(MemTxError::LengthOverflow(len))?;
    if end > bytes.len() {
        return Err(MemTxError::UnexpectedEof);
    }
    let slice = &bytes[*cursor..end];
    *cursor = end;
    Ok(slice)
}

#[cfg(test)]
mod tests {
    use super::{
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
}
