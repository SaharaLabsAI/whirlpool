//! TST-1: WhirlpoolProvider satisfies RpcModuleBuilder provider bounds
//! Extended in Task 05 to cover ChainSpecProvider, AccountReader, and CanonStateSubscriptions.

use alloy_primitives::Address;
use reth_chain_state::CanonStateSubscriptions;
use reth_chainspec::{ChainSpec, ChainSpecProvider};
use reth_ethereum_primitives::EthPrimitives;
use reth_rpc_builder::RpcModuleBuilder;
use reth_storage_api::AccountReader;
use rpc_eth::provider::WhirlpoolProvider;
use std::sync::Arc;

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
            + AccountReader
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

/// TST-1b: ChainSpecProvider returns a valid ChainSpec
#[test]
fn chain_spec_provider_returns_spec() {
    let (provider, _tmp) = test_provider();
    let spec = provider.chain_spec();
    // Default ChainSpec is mainnet (chain_id = 1) unless overridden
    assert!(
        spec.chain().id() > 0,
        "chain spec should have a valid chain id"
    );
}

/// TST-1c: AccountReader returns None for unknown address on empty DB
#[test]
fn account_reader_returns_none_for_unknown() {
    let (provider, _tmp) = test_provider();
    let addr = Address::ZERO;
    let result = provider
        .basic_account(&addr)
        .expect("basic_account should not error on empty db");
    assert!(result.is_none(), "unknown address should return None");
}

/// TST-1d: CanonStateSubscriptions yields a receiver
#[test]
fn canon_state_subscriptions_yields_receiver() {
    let (provider, _tmp) = test_provider();
    let _notifications: reth_chain_state::CanonStateNotifications<EthPrimitives> =
        provider.subscribe_to_canonical_state();
    // If this compiles and doesn't panic, the subscription channel is wired.
}

/// Helper: build a WhirlpoolProvider backed by a temporary MDBX database.
fn test_provider() -> (WhirlpoolProvider, tempfile::TempDir) {
    use app_evm_state::RethStateDb;

    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db = RethStateDb::open(tmp.path()).expect("failed to open MDBX");
    let chain_spec = Arc::new(ChainSpec::default());
    let provider = WhirlpoolProvider::new(Arc::new(db), chain_spec);
    (provider, tmp)
}
