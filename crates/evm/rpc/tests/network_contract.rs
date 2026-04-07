//! TST-3: WhirlpoolNetwork satisfies RPC builder network bounds.

use reth_ethereum_primitives::EthPrimitives;
use reth_network_api::{NetworkInfo, Peers};
use reth_rpc_builder::RpcModuleBuilder;
use rpc_eth::{network::WhirlpoolNetwork, pool::WhirlpoolTxPool, provider::WhirlpoolProvider};

/// Type-level assertion that WhirlpoolNetwork satisfies the network bounds
/// required by RpcModuleBuilder: NetworkInfo + Peers + Clone + 'static
fn _assert_network_bounds() {
    fn assert_bounds<N>()
    where
        N: NetworkInfo + Peers + Clone + Send + Sync + 'static,
    {
    }
    assert_bounds::<WhirlpoolNetwork>();
}

#[test]
fn network_satisfies_rpc_builder_bounds() {
    let _ = std::any::TypeId::of::<
        RpcModuleBuilder<
            EthPrimitives,
            WhirlpoolProvider,
            WhirlpoolTxPool,
            WhirlpoolNetwork,
            (),
            (),
        >,
    >();
    _assert_network_bounds();
}

#[test]
fn chain_id_round_trips() {
    let net = WhirlpoolNetwork::new(42);
    assert_eq!(net.chain_id(), 42);
}

#[test]
fn is_not_syncing() {
    let net = WhirlpoolNetwork::new(1);
    assert!(!net.is_syncing());
    assert!(!net.is_initially_syncing());
}

#[tokio::test]
async fn network_status_returns_ok() {
    let net = WhirlpoolNetwork::new(1);
    let status = net.network_status().await;
    assert!(status.is_ok(), "network_status should return Ok");
}

#[tokio::test]
async fn get_all_peers_returns_empty() {
    let net = WhirlpoolNetwork::new(1);
    let peers = net
        .get_all_peers()
        .await
        .expect("get_all_peers should not error");
    assert!(peers.is_empty(), "no peers in single-node system");
}
