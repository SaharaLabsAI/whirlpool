#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MemTxError {
    #[error("unsupported mem tx version: {0}")]
    UnsupportedVersion(u8),
    #[error("unsupported signature scheme: {0}")]
    UnsupportedSignatureScheme(u8),
    #[error("signer must not be empty")]
    EmptySigner,
    #[error("personality id must not be empty")]
    EmptyPersonalityId,
    #[error("markdown must not be empty")]
    EmptyMarkdown,
    #[error("signature must not be empty")]
    EmptySignature,
    #[error("signer length {len} exceeds limit {max}")]
    SignerTooLarge { len: usize, max: usize },
    #[error("personality id length {len} exceeds limit {max}")]
    PersonalityIdTooLarge { len: usize, max: usize },
    #[error("markdown length {len} exceeds limit {max}")]
    MarkdownTooLarge { len: usize, max: usize },
    #[error("signature length {len} exceeds limit {max}")]
    SignatureTooLarge { len: usize, max: usize },
    #[error("markdown bytes must be valid UTF-8")]
    InvalidUtf8Markdown,
    #[error("markdown hash mismatch")]
    MarkdownHashMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
    },
    #[error("unexpected end of payload")]
    UnexpectedEof,
    #[error("payload has {trailing} trailing bytes")]
    TrailingBytes { trailing: usize },
    #[error("encoded length overflow for {0} bytes")]
    LengthOverflow(usize),
}
