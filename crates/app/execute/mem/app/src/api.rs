use crate::{FinalizedPersonalityWrite, MemTxError, PersonalityMarkdownTx};

pub fn decode_personality_tx(bytes: &[u8]) -> Result<PersonalityMarkdownTx, MemTxError> {
    PersonalityMarkdownTx::decode(bytes)
}

pub fn derive_finalized_write(bytes: &[u8]) -> Result<FinalizedPersonalityWrite, MemTxError> {
    PersonalityMarkdownTx::decode(bytes)?.finalized_write()
}
