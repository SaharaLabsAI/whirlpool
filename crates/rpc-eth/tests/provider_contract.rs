//! TST-1: WhirlpoolProvider satisfies RpcModuleBuilder provider bounds

use reth_chainspec::ChainSpecProvider;
use reth_ethereum_primitives::EthPrimitives;
use reth_rpc_builder::RpcModuleBuilder;
use rpc_eth::provider::WhirlpoolProvider;

/// Type-level assertion that WhirlpoolProvider satisfies the provider bounds
/// required by RpcModuleBuilder
fn _assert_provider_bounds() {
    fn assert_bounds<P>()
    where
        P: reth_storage_api::BlockReaderIdExt
            + ChainSpecProvider
            + reth_storage_api::StateProviderFactory
            + reth_chain_state::CanonStateSubscriptions
            + reth_storage_api::StageCheckpointReader
            + Clone
            + Send
            + Sync
            + Unpin
            + 'static,
    {
    }
    assert_bounds::<WhirlpoolProvider>();
}

#[test]
fn provider_satisfies_rpc_node_core_bounds() {
    // If this compiles, the bounds are satisfied.
    let _ = std::any::TypeId::of::<
        RpcModuleBuilder<EthPrimitives, WhirlpoolProvider, (), (), (), ()>,
    >();
    _assert_provider_bounds();
}
