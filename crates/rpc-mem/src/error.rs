#[derive(Debug, thiserror::Error)]
pub enum RpcMemError {
    #[error("unsupported mem tx version: {0}")]
    UnsupportedVersion(u8),
    #[error("unsupported signature scheme: {0}")]
    UnsupportedSignatureScheme(String),
    #[error("{field} must be a 0x-prefixed hex string")]
    InvalidHexPrefix { field: &'static str },
    #[error("{field} must decode from hex: {source}")]
    InvalidHex {
        field: &'static str,
        #[source]
        source: hex::FromHexError,
    },
    #[error(transparent)]
    Mem(#[from] app_mem::MemTxError),
    #[error(transparent)]
    JsonRpseeCore(#[from] jsonrpsee::core::ClientError),
    #[error(transparent)]
    JsonRpseeRegister(#[from] jsonrpsee::core::RegisterMethodError),
    #[error(transparent)]
    JsonRpseeTransport(#[from] std::io::Error),
}
