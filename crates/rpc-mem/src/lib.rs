mod error;

pub use error::RpcMemError;

use std::net::SocketAddr;
use std::sync::Arc;

use app::traits::TxSource;
use app_mem::{PersonalityMarkdownTx, SignatureScheme, SUPPORTED_PERSONALITY_TX_VERSION};
use jsonrpsee::core::RpcResult;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use state::{PersonalityStorage, StoredPersonality};

type PersonalityLookup =
    dyn Fn(&[u8]) -> Result<Option<StoredPersonality>, RpcMemError> + Send + Sync;

pub trait MemoryTxService: Send + Sync {
    fn submit_personality(
        &self,
        request: SubmitPersonalityRequest,
    ) -> Result<[u8; 32], RpcMemError>;

    fn get_personality(
        &self,
        personality_id: Vec<u8>,
    ) -> Result<Option<StoredPersonality>, RpcMemError>;
}

#[derive(Clone)]
pub struct TxSourceMemoryTxService {
    tx_source: Arc<dyn TxSource>,
    personality_lookup: Arc<PersonalityLookup>,
}

impl TxSourceMemoryTxService {
    pub fn new(tx_source: Arc<dyn TxSource>) -> Self {
        Self {
            tx_source,
            personality_lookup: Arc::new(|_| Err(RpcMemError::ReadCapabilityUnavailable)),
        }
    }

    pub fn with_personality_storage<PS>(
        tx_source: Arc<dyn TxSource>,
        personality_storage: Arc<PS>,
    ) -> Self
    where
        PS: PersonalityStorage + 'static,
        PS::Error: std::fmt::Display,
    {
        Self {
            tx_source,
            personality_lookup: Arc::new(move |personality_id| {
                personality_storage
                    .get_latest(personality_id)
                    .map_err(|err| RpcMemError::PersonalityRead(err.to_string()))
            }),
        }
    }
}

impl MemoryTxService for TxSourceMemoryTxService {
    fn submit_personality(
        &self,
        request: SubmitPersonalityRequest,
    ) -> Result<[u8; 32], RpcMemError> {
        let tx = request.into_tx()?;
        let encoded = tx.encode()?;
        let tx_hash = tx.tx_hash()?;
        self.tx_source.push(encoded);
        Ok(tx_hash)
    }

    fn get_personality(
        &self,
        personality_id: Vec<u8>,
    ) -> Result<Option<StoredPersonality>, RpcMemError> {
        (self.personality_lookup)(&personality_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitPersonalityRequest {
    pub version: u8,
    pub signer: String,
    pub personality_id: String,
    pub nonce: u64,
    pub markdown: String,
    pub signature_scheme: String,
    pub signature: String,
}

impl SubmitPersonalityRequest {
    fn into_tx(self) -> Result<PersonalityMarkdownTx, RpcMemError> {
        if self.version != SUPPORTED_PERSONALITY_TX_VERSION {
            return Err(RpcMemError::UnsupportedVersion(self.version));
        }

        let signature_scheme = parse_signature_scheme(&self.signature_scheme)?;
        let signer = decode_hex_field("signer", &self.signer)?;
        let personality_id = decode_hex_field("personality_id", &self.personality_id)?;
        let signature = decode_hex_field("signature", &self.signature)?;

        let tx = PersonalityMarkdownTx::new(
            signer,
            personality_id,
            self.nonce,
            self.markdown.into_bytes(),
            signature_scheme,
            signature,
        );
        tx.validate()?;
        Ok(tx)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitPersonalityResponse {
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPersonalityRequest {
    pub personality_id: String,
}

impl GetPersonalityRequest {
    fn personality_id_bytes(&self) -> Result<Vec<u8>, RpcMemError> {
        decode_hex_field("personality_id", &self.personality_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPersonalityResponse {
    pub tx_hash: String,
    pub block_height: u64,
    pub signer: String,
    pub personality_id: String,
    pub nonce: u64,
    pub markdown: String,
    pub markdown_hash: String,
}

impl TryFrom<StoredPersonality> for GetPersonalityResponse {
    type Error = RpcMemError;

    fn try_from(entry: StoredPersonality) -> Result<Self, Self::Error> {
        Ok(Self {
            tx_hash: encode_hex(&entry.tx_hash),
            block_height: entry.block_height,
            signer: encode_hex(&entry.signer),
            personality_id: encode_hex(&entry.personality_id),
            nonce: entry.nonce,
            markdown: String::from_utf8(entry.markdown)
                .map_err(RpcMemError::InvalidStoredMarkdown)?,
            markdown_hash: encode_hex(&entry.markdown_hash),
        })
    }
}

pub async fn start_rpc_server(
    service: Arc<dyn MemoryTxService>,
    addr: SocketAddr,
) -> Result<(ServerHandle, SocketAddr), RpcMemError> {
    let server = ServerBuilder::default().build(addr).await?;
    let local_addr = server.local_addr()?;
    let mut module = RpcModule::new(service);

    module.register_method(
        "mem_submitPersonality",
        |params, service, _| -> RpcResult<SubmitPersonalityResponse> {
            let request: SubmitPersonalityRequest = params.one()?;
            let tx_hash = service
                .submit_personality(request)
                .map_err(rpc_error_from_service)?;

            Ok(SubmitPersonalityResponse {
                tx_hash: encode_hex(&tx_hash),
            })
        },
    )?;

    module.register_method(
        "mem_getPersonality",
        |params, service, _| -> RpcResult<Option<GetPersonalityResponse>> {
            let request: GetPersonalityRequest = params.one()?;
            let personality_id = request
                .personality_id_bytes()
                .map_err(rpc_error_from_service)?;
            let response = service
                .get_personality(personality_id)
                .map_err(rpc_error_from_service)?
                .map(GetPersonalityResponse::try_from)
                .transpose()
                .map_err(rpc_error_from_service)?;

            Ok(response)
        },
    )?;

    let handle = server.start(module);
    Ok((handle, local_addr))
}

fn encode_hex(value: &[u8]) -> String {
    format!("0x{}", hex::encode(value))
}

fn parse_signature_scheme(value: &str) -> Result<SignatureScheme, RpcMemError> {
    match value {
        "raw_secp256k1" => Ok(SignatureScheme::RawSecp256k1),
        other => Err(RpcMemError::UnsupportedSignatureScheme(other.to_string())),
    }
}

fn decode_hex_field(field: &'static str, value: &str) -> Result<Vec<u8>, RpcMemError> {
    let stripped = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .ok_or(RpcMemError::InvalidHexPrefix { field })?;
    hex::decode(stripped).map_err(|source| RpcMemError::InvalidHex { field, source })
}

fn rpc_error_from_service(error: RpcMemError) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32001, error.to_string(), None::<()>)
}
