use std::sync::{Arc, Mutex};

use jsonrpsee::core::{client::ClientT, rpc_params};
use jsonrpsee::http_client::HttpClientBuilder;
use rpc_mem::{
    start_rpc_server, GetPersonalityRequest, GetPersonalityResponse, GetTransactionByHashRequest,
    GetTransactionByHashResponse, MemoryTxService, RpcMemError, SubmitPersonalityRequest,
    TxSourceMemoryTxService,
};
use state::{PersonalityStorage, StoredPersonality};
use state_memory::InMemoryPersonalityStorage;

use app::traits::TxSource;

#[derive(Debug)]
struct FakeMemoryTxService {
    lookup_result: Mutex<Option<StoredPersonality>>,
    lookup_calls: Mutex<Vec<Vec<u8>>>,
    tx_hash_lookup_result: Mutex<Option<StoredPersonality>>,
    tx_hash_lookup_calls: Mutex<Vec<[u8; 32]>>,
}

impl FakeMemoryTxService {
    fn new(lookup_result: Option<StoredPersonality>) -> Self {
        Self {
            lookup_result: Mutex::new(lookup_result),
            lookup_calls: Mutex::new(Vec::new()),
            tx_hash_lookup_result: Mutex::new(None),
            tx_hash_lookup_calls: Mutex::new(Vec::new()),
        }
    }

    fn with_tx_hash_lookup(
        lookup_result: Option<StoredPersonality>,
        tx_hash_lookup_result: Option<StoredPersonality>,
    ) -> Self {
        Self {
            lookup_result: Mutex::new(lookup_result),
            lookup_calls: Mutex::new(Vec::new()),
            tx_hash_lookup_result: Mutex::new(tx_hash_lookup_result),
            tx_hash_lookup_calls: Mutex::new(Vec::new()),
        }
    }

    fn lookup_calls(&self) -> Vec<Vec<u8>> {
        self.lookup_calls
            .lock()
            .expect("poisoned lookup call mutex")
            .clone()
    }

    fn tx_hash_lookup_calls(&self) -> Vec<[u8; 32]> {
        self.tx_hash_lookup_calls
            .lock()
            .expect("poisoned tx hash lookup call mutex")
            .clone()
    }
}

impl MemoryTxService for FakeMemoryTxService {
    fn submit_personality(
        &self,
        _request: SubmitPersonalityRequest,
    ) -> Result<[u8; 32], RpcMemError> {
        Ok([0x11; 32])
    }

    fn get_personality(
        &self,
        personality_id: Vec<u8>,
    ) -> Result<Option<StoredPersonality>, RpcMemError> {
        self.lookup_calls
            .lock()
            .expect("poisoned lookup call mutex")
            .push(personality_id);
        Ok(self
            .lookup_result
            .lock()
            .expect("poisoned lookup result mutex")
            .clone())
    }

    fn get_transaction_by_hash(
        &self,
        tx_hash: [u8; 32],
    ) -> Result<Option<StoredPersonality>, RpcMemError> {
        self.tx_hash_lookup_calls
            .lock()
            .expect("poisoned tx hash lookup call mutex")
            .push(tx_hash);
        Ok(self
            .tx_hash_lookup_result
            .lock()
            .expect("poisoned tx hash lookup result mutex")
            .clone())
    }
}

fn sample_stored_personality() -> StoredPersonality {
    StoredPersonality {
        tx_hash: [0xab; 32],
        block_height: 42,
        version: 1,
        signer: vec![0xcd; 20],
        personality_id: vec![0xef; 16],
        nonce: 7,
        markdown: "# Final persona\nBe precise.".as_bytes().to_vec(),
        markdown_hash: [0x34; 32],
        signature_scheme: 1,
        signature: vec![0x11; 65],
    }
}

fn sample_request() -> GetPersonalityRequest {
    GetPersonalityRequest {
        personality_id: "0xefefefefefefefefefefefefefefefef".to_string(),
    }
}

#[derive(Debug, Default)]
struct RecordingTxSource;

impl TxSource for RecordingTxSource {
    fn push(&self, _tx: Vec<u8>) {}

    fn pending(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

#[tokio::test]
async fn rpc_server_returns_latest_finalized_personality_entry() {
    let service = Arc::new(FakeMemoryTxService::new(Some(sample_stored_personality())));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let response: Option<GetPersonalityResponse> = client
        .request("mem_getPersonality", rpc_params![sample_request()])
        .await
        .expect("rpc get personality should succeed");

    assert_eq!(
        response,
        Some(GetPersonalityResponse {
            tx_hash: format!("0x{}", "ab".repeat(32)),
            block_height: 42,
            signer: format!("0x{}", "cd".repeat(20)),
            personality_id: format!("0x{}", "ef".repeat(16)),
            nonce: 7,
            markdown: "# Final persona\nBe precise.".to_string(),
            markdown_hash: format!("0x{}", "34".repeat(32)),
        })
    );
    assert_eq!(service_handle.lookup_calls(), vec![vec![0xef; 16]]);

    handle.stop().expect("server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn rpc_server_returns_null_when_personality_is_missing() {
    let service = Arc::new(FakeMemoryTxService::new(None));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let response: Option<GetPersonalityResponse> = client
        .request("mem_getPersonality", rpc_params![sample_request()])
        .await
        .expect("missing personality lookup should succeed with null result");

    assert_eq!(response, None);
    assert_eq!(service_handle.lookup_calls(), vec![vec![0xef; 16]]);

    handle.stop().expect("server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn rpc_server_rejects_malformed_personality_hex_without_calling_service() {
    let service = Arc::new(FakeMemoryTxService::new(Some(sample_stored_personality())));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let err = client
        .request::<Option<GetPersonalityResponse>, _>(
            "mem_getPersonality",
            rpc_params![GetPersonalityRequest {
                personality_id: "efef".to_string(),
            }],
        )
        .await
        .expect_err("invalid personality id must fail validation");

    assert!(err
        .to_string()
        .contains("personality_id must be a 0x-prefixed hex string"));
    assert!(service_handle.lookup_calls().is_empty());

    handle.stop().expect("server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn tx_source_service_reads_latest_finalized_personality_from_storage() {
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource);
    let personality_storage = Arc::new(InMemoryPersonalityStorage::new());
    personality_storage
        .put(sample_stored_personality())
        .expect("storing finalized personality should succeed");

    let service = TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage);

    let response = service
        .get_personality(vec![0xef; 16])
        .expect("storage-backed lookup should succeed");

    assert_eq!(response, Some(sample_stored_personality()));
}

#[tokio::test]
async fn tx_source_service_returns_none_for_missing_storage_entry() {
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource);
    let personality_storage = Arc::new(InMemoryPersonalityStorage::new());

    let service = TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage);

    let response = service
        .get_personality(vec![0xaa; 16])
        .expect("missing storage lookup should succeed");

    assert_eq!(response, None);
}

#[tokio::test]
async fn tx_source_service_reads_finalized_tx_by_hash_from_storage() {
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource);
    let personality_storage = Arc::new(InMemoryPersonalityStorage::new());
    let sample = sample_stored_personality();
    personality_storage
        .put(sample.clone())
        .expect("storing finalized personality should succeed");

    let service = TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage);

    let response = service
        .get_transaction_by_hash(sample.tx_hash)
        .expect("storage-backed tx hash lookup should succeed");

    assert_eq!(response, Some(sample));
}

#[tokio::test]
async fn tx_source_service_returns_none_for_missing_tx_hash_storage_entry() {
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource);
    let personality_storage = Arc::new(InMemoryPersonalityStorage::new());

    let service = TxSourceMemoryTxService::with_personality_storage(tx_source, personality_storage);

    let response = service
        .get_transaction_by_hash([0x55; 32])
        .expect("missing tx hash lookup should succeed");

    assert_eq!(response, None);
}

#[tokio::test]
async fn rpc_server_returns_finalized_tx_by_hash() {
    let sample = sample_stored_personality();
    let service = Arc::new(FakeMemoryTxService::with_tx_hash_lookup(
        Some(sample.clone()),
        Some(sample),
    ));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let response: Option<GetTransactionByHashResponse> = client
        .request(
            "mem_getTransactionByHash",
            rpc_params![GetTransactionByHashRequest {
                tx_hash: format!("0x{}", "ab".repeat(32)),
            }],
        )
        .await
        .expect("rpc get transaction by hash should succeed");

    assert_eq!(
        response,
        Some(GetTransactionByHashResponse {
            tx_hash: format!("0x{}", "ab".repeat(32)),
            block_height: 42,
            version: 1,
            signer: format!("0x{}", "cd".repeat(20)),
            personality_id: format!("0x{}", "ef".repeat(16)),
            nonce: 7,
            markdown: "# Final persona\nBe precise.".to_string(),
            markdown_hash: format!("0x{}", "34".repeat(32)),
            signature_scheme: "raw_secp256k1".to_string(),
            signature: format!("0x{}", "11".repeat(65)),
        })
    );
    assert_eq!(service_handle.tx_hash_lookup_calls(), vec![[0xab; 32]]);

    handle.stop().expect("server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn rpc_server_returns_null_when_tx_hash_is_missing() {
    let sample = sample_stored_personality();
    let service = Arc::new(FakeMemoryTxService::with_tx_hash_lookup(Some(sample), None));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let response: Option<GetTransactionByHashResponse> = client
        .request(
            "mem_getTransactionByHash",
            rpc_params![GetTransactionByHashRequest {
                tx_hash: format!("0x{}", "bb".repeat(32)),
            }],
        )
        .await
        .expect("missing tx hash lookup should succeed with null result");

    assert_eq!(response, None);
    assert_eq!(service_handle.tx_hash_lookup_calls(), vec![[0xbb; 32]]);

    handle.stop().expect("server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn rpc_server_rejects_malformed_tx_hash_without_calling_service() {
    let service = Arc::new(FakeMemoryTxService::new(Some(sample_stored_personality())));
    let service_handle = service.clone();
    let service: Arc<dyn MemoryTxService> = service;
    let (handle, addr) = start_rpc_server(service, "127.0.0.1:0".parse().unwrap())
        .await
        .expect("server should start");

    let client = HttpClientBuilder::default()
        .build(format!("http://{addr}"))
        .expect("client should build");
    let err = client
        .request::<Option<GetTransactionByHashResponse>, _>(
            "mem_getTransactionByHash",
            rpc_params![GetTransactionByHashRequest {
                tx_hash: "ab".to_string(),
            }],
        )
        .await
        .expect_err("invalid tx hash must fail validation");

    assert!(err
        .to_string()
        .contains("tx_hash must be a 0x-prefixed hex string"));
    assert!(service_handle.tx_hash_lookup_calls().is_empty());

    handle.stop().expect("server should stop");
    handle.stopped().await;
}
