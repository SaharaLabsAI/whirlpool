use crate::{
    hash::compute_payload_hash, FinalizedPersonalityWrite, MemTxError, PersonalityMarkdownTx,
};

impl PersonalityMarkdownTx {
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
