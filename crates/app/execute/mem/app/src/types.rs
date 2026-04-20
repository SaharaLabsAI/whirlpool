use crate::MemTxError;

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
