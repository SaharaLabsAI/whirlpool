use crate::{
    codec::{decode_personality_tx as decode_raw_personality_tx, encode_personality_tx},
    validation::validate_personality_tx,
    MemTxError, PersonalityMarkdownTx,
};

impl PersonalityMarkdownTx {
    pub fn encode(&self) -> Result<Vec<u8>, MemTxError> {
        self.validate()?;
        encode_personality_tx(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MemTxError> {
        let tx = decode_raw_personality_tx(bytes)?;
        tx.validate()?;
        Ok(tx)
    }

    pub fn validate(&self) -> Result<(), MemTxError> {
        validate_personality_tx(self)
    }
}
