mod api;
mod codec;
mod error;
mod hash;
mod tx_codec_api;
mod tx_constructor;
mod tx_finalize_api;
mod types;
mod validation;

pub use api::{decode_personality_tx, derive_finalized_write};
pub use error::MemTxError;
pub use hash::compute_markdown_hash;
pub use types::{
    FinalizedPersonalityWrite, PersonalityMarkdownTx, SignatureScheme, MAX_IDENTITY_BYTES,
    MAX_PERSONALITY_MARKDOWN_BYTES, MAX_SIGNATURE_BYTES, PERSONALITY_MARKDOWN_TYPE_ID,
    SUPPORTED_PERSONALITY_TX_VERSION,
};

#[cfg(test)]
mod tests;
