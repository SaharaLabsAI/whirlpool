use std::sync::{Arc, Mutex};

use jsonrpsee::core::{client::ClientT, rpc_params};
use jsonrpsee::http_client::HttpClientBuilder;
use rpc_mem::{
    GetPersonalityRequest, GetPersonalityResponse, MemoryTxService, RpcMemError,
    SubmitPersonalityRequest, start_rpc_server,
};
use state::StoredPersonality;

#[derive(Debug)]
struct FakeMemoryTxService {
    lookup_result: Mutex<Option<StoredPersonality>>,
    lookup_calls: Mutex<Vec<Vec<u8>>>,
}

impl FakeMemoryTxService {
    fn new(lookup_result: Option<StoredPersonality>) -> Self {
        Self {
            lookup_result: Mutex::new(lookup_result),
            lookup_calls: Mutex::new(Vec::new()),
        }
    }

    fn lookup_calls(&self) -> Vec<Vec<u8>> {
        self.lookup_calls
            .lock()
            .expect("poisoned lookup call mutex")
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

    fn get_personality(&self, personality_id: Vec<u8>) -> Result<Option<StoredPersonality>, RpcMemError> {
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
}

fn sample_stored_personality() -> StoredPersonality {
    StoredPersonality {
        tx_hash: [0xab; 32],
        block_height: 42,
        signer: vec![0xcd; 20],
        personality_id: vec![0xef; 16],
        nonce: 7,
        markdown: "# Final persona\nBe precise.".as_bytes().to_vec(),
        markdown_hash: [0x34; 32],
    }
}

fn sample_request() -> GetPersonalityRequest {
    GetPersonalityRequest {
        personality_id: "0xefefefefefefefefefefefefefefefef".to_string(),
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

    assert!(err.to_string().contains("personality_id must be a 0x-prefixed hex string"));
    assert!(service_handle.lookup_calls().is_empty());

    handle.stop().expect("server should stop");
    handle.stopped().await;
}
