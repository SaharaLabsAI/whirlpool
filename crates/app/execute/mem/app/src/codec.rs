use crate::{MemTxError, PersonalityMarkdownTx, SignatureScheme};

pub fn encode_personality_tx(tx: &PersonalityMarkdownTx) -> Result<Vec<u8>, MemTxError> {
    let mut encoded = Vec::with_capacity(encoded_len(tx));
    encoded.push(tx.version);
    put_len_prefixed(&mut encoded, &tx.signer)?;
    put_len_prefixed(&mut encoded, &tx.personality_id)?;
    encoded.extend_from_slice(&tx.nonce.to_le_bytes());
    put_len_prefixed(&mut encoded, &tx.markdown_bytes)?;
    encoded.extend_from_slice(&tx.markdown_hash);
    encoded.push(tx.signature_scheme.to_wire());
    put_len_prefixed(&mut encoded, &tx.signature)?;
    Ok(encoded)
}

pub fn decode_personality_tx(bytes: &[u8]) -> Result<PersonalityMarkdownTx, MemTxError> {
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

    Ok(PersonalityMarkdownTx {
        version,
        signer,
        personality_id,
        nonce,
        markdown_bytes,
        markdown_hash,
        signature_scheme,
        signature,
    })
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
