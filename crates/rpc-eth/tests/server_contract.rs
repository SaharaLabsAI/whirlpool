use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use app::traits::TxSource;
use jsonrpsee::core::{client::ClientT, rpc_params};
use reth_chainspec::ChainSpec;
use rpc_eth::server::start_rpc_server;
use state_reth::RethStateDb;

#[derive(Debug, Default)]
struct RecordingTxSource {
    pending: Mutex<Vec<Vec<u8>>>,
}

impl TxSource for RecordingTxSource {
    fn push(&self, tx: Vec<u8>) {
        self.pending.lock().expect("poisoned tx source mutex").push(tx);
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        self.pending.lock().expect("poisoned tx source mutex").clone()
    }
}

#[tokio::test]
async fn server_starts_and_returns_local_address() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let state_db = Arc::new(RethStateDb::open(tmp.path()).expect("failed to open MDBX"));
    let chain_spec = Arc::new(ChainSpec::default());
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource::default());

    let (server, local_addr) = start_rpc_server(
        state_db,
        chain_spec,
        tx_source,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    )
    .await
    .expect("server should start");

    assert_ne!(local_addr.port(), 0, "port 0 should be replaced by a bound port");
    assert_eq!(server.http_local_addr(), Some(local_addr));

    server.stop().expect("server should stop cleanly");
}

#[tokio::test]
async fn server_responds_to_eth_chain_id() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let state_db = Arc::new(RethStateDb::open(tmp.path()).expect("failed to open MDBX"));
    let chain_spec = Arc::new(ChainSpec::default());
    let expected_chain_id = chain_spec.chain().id();
    let tx_source: Arc<dyn TxSource> = Arc::new(RecordingTxSource::default());

    let (server, _local_addr) = start_rpc_server(
        state_db,
        chain_spec,
        tx_source,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    )
    .await
    .expect("server should start");

    let client = server.http_client().expect("http client should be available");
    let chain_id: Option<String> = client
        .request("eth_chainId", rpc_params![])
        .await
        .expect("eth_chainId should succeed");

    assert_eq!(chain_id, Some(format!("0x{expected_chain_id:x}")));

    server.stop().expect("server should stop cleanly");
}
