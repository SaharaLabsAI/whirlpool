use sha2::{Digest, Sha256};

pub fn compute_markdown_hash(markdown_bytes: &[u8]) -> [u8; 32] {
    compute_payload_hash(markdown_bytes)
}

pub fn compute_payload_hash(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}
