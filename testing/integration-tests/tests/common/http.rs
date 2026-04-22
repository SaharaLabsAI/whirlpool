use std::net::SocketAddr;
use std::sync::OnceLock;

use reqwest::Client;

pub fn test_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

pub fn rpc_req(method: &str, params: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
}

fn rpc_http_url(rpc_addr: SocketAddr) -> String {
    format!("http://{rpc_addr}")
}

pub async fn post_json_to_addr(
    client: &Client,
    rpc_addr: SocketAddr,
    body: serde_json::Value,
) -> serde_json::Value {
    let response = client
        .post(rpc_http_url(rpc_addr))
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|err| panic!("failed to send RPC request to {rpc_addr}: {err}"));

    response
        .json()
        .await
        .unwrap_or_else(|err| panic!("failed to decode RPC response from {rpc_addr}: {err}"))
}
