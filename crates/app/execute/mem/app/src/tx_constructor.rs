use crate::{
    compute_markdown_hash, PersonalityMarkdownTx, SignatureScheme, SUPPORTED_PERSONALITY_TX_VERSION,
};

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
}
