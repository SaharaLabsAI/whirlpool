use std::sync::{Arc, RwLock};

use alloy_primitives::Address;
use app_evm_execution::{EvmApplication, WhirlpoolEvmConfig};
use app_evm_state::InMemoryStateDb;
use app_primitives::EvmBlock;
use app_traits::{traits::Application, ApplicationAdapter, NoopTxSource};
use chainspec::genesis::{build_sahara_chain_spec_from, SaharaGenesisConfig};
use consensus::{traits::ConsensusApp, ConsensusError};
use revm::primitives::U256;
use validators_reader::{
    encode_validator_registry_storage, ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY,
};

const TEST_PROPOSER_FEE_RECIPIENT: Address = Address::new([
    0x70, 0x72, 0x6f, 0x70, 0x6f, 0x73, 0x65, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x73, 0x65, 0x61,
    0x6d, 0x2d, 0x30, 0x31,
]);

fn assert_application_impl<A: Application<Block = EvmBlock>>(_app: &A) {}

fn build_test_chain_spec() -> reth_chainspec::ChainSpec {
    build_sahara_chain_spec_from(SaharaGenesisConfig {
        simplex_validators: test_validator_entries(),
        ..SaharaGenesisConfig::default()
    })
}

fn test_validator_entries() -> Vec<ValidatorEntry> {
    vec![ValidatorEntry {
        consensus_pubkey: [0x11; 32],
        ethereum_address: TEST_PROPOSER_FEE_RECIPIENT,
    }]
}

fn seed_validator_registry(db: &mut InMemoryStateDb, entries: &[ValidatorEntry]) {
    for (slot, value) in encode_validator_registry_storage(entries) {
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            U256::from_be_bytes(slot.0),
            U256::from_be_bytes(value.0),
        );
    }
}

fn build_adapter() -> ApplicationAdapter<EvmApplication<InMemoryStateDb>> {
    let entries = test_validator_entries();
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    seed_validator_registry(&mut state_db.write().unwrap(), &entries);
    let evm_config = WhirlpoolEvmConfig::new(Arc::new(build_test_chain_spec()))
        .with_local_proposer_public_key(
            entries
                .first()
                .expect("test registry has a proposer")
                .consensus_pubkey,
        );
    let tx_source = Arc::new(NoopTxSource);
    let evm_app = EvmApplication::new(evm_config, state_db, tx_source);
    assert_application_impl(&evm_app);
    ApplicationAdapter::new(evm_app)
}

#[tokio::test]
async fn test_adapter_propose_success() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let proposed = adapter.propose(&genesis, 1).await;

    assert!(matches!(proposed, Some(EvmBlock { .. })));
}

#[tokio::test]
async fn test_adapter_propose_returns_some() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block = adapter
        .propose(&genesis, 1)
        .await
        .expect("adapter should wrap Application::propose Ok as Some");

    assert_eq!(block.height, 1);
    assert_eq!(block.parent_id, genesis.compute_id());
}

#[tokio::test]
async fn test_adapter_verify_success() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let verify_result = adapter.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_adapter_verify_failure() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let verify_result = adapter.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        verify_result,
        Err(ConsensusError::InvalidBlock(_))
    ));
}

#[tokio::test]
async fn test_error_propagation_through_adapter() {
    let adapter = build_adapter();

    let genesis = ConsensusApp::genesis(&adapter).await;
    let block1 = ConsensusApp::propose(&adapter, &genesis, 1)
        .await
        .expect("adapter propose should return Some");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let verify_result = ConsensusApp::verify(&adapter, &genesis, &tampered_block).await;
    assert!(matches!(
        verify_result,
        Err(ConsensusError::InvalidBlock(_))
    ));
}

#[tokio::test]
async fn test_state_root_mismatch_maps_to_consensus_invalid_block() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let adapter_verify = adapter.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        adapter_verify,
        Err(ConsensusError::InvalidBlock(_))
    ));
}
