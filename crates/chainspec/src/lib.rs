use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Address, B256, U256};
use app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY;
use evm_precompiles::{
    current_epoch_storage_slot, encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_blocks_storage_slot, epoch_system_tx_sender, next_epoch_block_storage_slot,
    EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI,
};
use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use std::collections::BTreeMap;
use validators::{
    decode_validator_registry_storage_opt, encode_ethereum_address_storage_value,
    encode_validator_registry_storage, ValidatorEntry, ValidatorRegistryError,
    SIMPLEX_VALIDATORS_REGISTRY,
};

mod native_token;
pub use native_token::{
    sahara_hard_cap_base_units, total_allocated_supply, validate_genesis_alloc, NativeTokenError,
    SAHARA_DECIMALS, SAHARA_HARD_CAP_BASE_UNITS_U128, SAHARA_HARD_CAP_TOKENS,
};

pub const SAHARA_CHAIN_ID: u64 = 313_371;

pub fn build_sahara_chain_spec() -> ChainSpec {
    try_build_sahara_chain_spec()
        .expect("default Sahara chain spec should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec() -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc(BTreeMap::new())
}

/// Build the Sahara chain spec with pre-funded genesis accounts.
///
/// This is useful for integration tests that need accounts with ETH balances
/// at genesis to submit transactions.
pub fn build_sahara_chain_spec_with_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc(alloc)
        .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec_with_alloc(
    alloc: BTreeMap<Address, GenesisAccount>,
) -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        BTreeMap::new(),
        Vec::new(),
    )
}

pub fn build_sahara_chain_spec_with_alloc_and_fee_recipients(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        validator_fee_recipients,
        Vec::new(),
    )
    .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
    simplex_validators: Vec<ValidatorEntry>,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        validator_fee_recipients,
        simplex_validators,
    )
    .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec_with_alloc_and_fee_recipients(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
) -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        validator_fee_recipients,
        Vec::new(),
    )
}

pub fn try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
    mut alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
    simplex_validators: Vec<ValidatorEntry>,
) -> Result<ChainSpec, NativeTokenError> {
    if !validator_fee_recipients.is_empty() {
        let account = alloc
            .entry(VALIDATOR_FEE_RECIPIENTS_REGISTRY)
            .or_insert_with(|| GenesisAccount {
                balance: U256::ZERO,
                ..GenesisAccount::default()
            });

        let storage = account.storage.get_or_insert_with(BTreeMap::new);
        for (validator_public_key, fee_recipient) in validator_fee_recipients {
            storage.insert(
                B256::from(validator_public_key),
                encode_ethereum_address_storage_value(fee_recipient),
            );
        }
    }

    if !simplex_validators.is_empty() {
        let account = alloc
            .entry(SIMPLEX_VALIDATORS_REGISTRY)
            .or_insert_with(|| GenesisAccount {
                balance: U256::ZERO,
                ..GenesisAccount::default()
            });
        account.storage = Some(encode_validator_registry_storage(&simplex_validators));
    }

    seed_epoch_precompile_genesis_state(&mut alloc);

    validate_genesis_alloc(&alloc)?;

    Ok(ChainSpecBuilder::default()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(Genesis {
            gas_limit: 30_000_000,
            difficulty: U256::ZERO,
            alloc,
            ..Default::default()
        })
        .cancun_activated()
        .build())
}

fn seed_epoch_precompile_genesis_state(alloc: &mut BTreeMap<Address, GenesisAccount>) {
    let account = alloc
        .entry(EPOCH_PRECOMPILE_ADDRESS)
        .or_insert_with(|| GenesisAccount {
            balance: U256::ZERO,
            ..GenesisAccount::default()
        });
    let storage = account.storage.get_or_insert_with(BTreeMap::new);
    storage.insert(current_epoch_storage_slot(), encode_u64_storage_value(0));
    storage.insert(
        epoch_blocks_storage_slot(),
        encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT),
    );
    storage.insert(
        next_epoch_block_storage_slot(),
        encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT),
    );
    storage.insert(
        evm_precompiles::epoch_start_block_storage_slot(0),
        encode_epoch_start_block_storage_value(0),
    );

    let sender = epoch_system_tx_sender();
    let sender_account = alloc.entry(sender).or_insert_with(|| GenesisAccount {
        balance: U256::ZERO,
        ..GenesisAccount::default()
    });
    sender_account.balance = sender_account
        .balance
        .checked_add(U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI))
        .expect("epoch system sender balance seed should not overflow");
    sender_account.nonce = Some(0);
}

pub fn try_simplex_validators_from_chain_spec(
    chain_spec: &ChainSpec,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    decode_validator_registry_storage_opt(
        chain_spec
            .genesis
            .alloc
            .get(&SIMPLEX_VALIDATORS_REGISTRY)
            .and_then(|account| account.storage.as_ref()),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_sahara_chain_spec,
        build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators,
        current_epoch_storage_slot, encode_epoch_start_block_storage_value, encode_u64_storage_value,
        epoch_blocks_storage_slot, epoch_system_tx_sender, next_epoch_block_storage_slot,
        sahara_hard_cap_base_units, try_build_sahara_chain_spec_with_alloc,
        try_simplex_validators_from_chain_spec, NativeTokenError, EPOCH_BLOCKS_DEFAULT,
        EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI, SAHARA_CHAIN_ID,
    };
    use alloy_genesis::GenesisAccount;
    use alloy_primitives::{address, Address, U256};
    use reth_chainspec::EthereumHardforks;
    use std::collections::BTreeMap;
    use validators::{ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY};

    #[test]
    fn test_build_sahara_chain_spec_values() {
        let spec = build_sahara_chain_spec();

        assert_eq!(spec.chain.id(), SAHARA_CHAIN_ID);
        assert_eq!(spec.genesis.gas_limit, 30_000_000);
        assert!(spec.is_cancun_active_at_timestamp(0));
    }

    #[test]
    fn chain_spec_builder_writes_epoch_precompile_genesis_state() {
        let spec = build_sahara_chain_spec();
        let account = spec
            .genesis
            .alloc
            .get(&EPOCH_PRECOMPILE_ADDRESS)
            .expect("epoch precompile account");
        let storage = account.storage.as_ref().expect("epoch storage");

        assert_eq!(
            storage.get(&current_epoch_storage_slot()),
            Some(&encode_u64_storage_value(0))
        );
        assert_eq!(
            storage.get(&epoch_blocks_storage_slot()),
            Some(&encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT))
        );
        assert_eq!(
            storage.get(&next_epoch_block_storage_slot()),
            Some(&encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT))
        );
        assert_eq!(
            storage.get(&evm_precompiles::epoch_start_block_storage_slot(0)),
            Some(&encode_epoch_start_block_storage_value(0))
        );
    }

    #[test]
    fn chain_spec_builder_seeds_epoch_system_sender_balance() {
        let spec = build_sahara_chain_spec();
        let sender = epoch_system_tx_sender();
        let account = spec
            .genesis
            .alloc
            .get(&sender)
            .expect("epoch system sender account");
        assert_eq!(
            account.balance,
            U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI)
        );
        assert_eq!(account.nonce, Some(0));
    }

    #[test]
    fn chain_spec_builder_writes_validator_registry() {
        let validators = vec![
            ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x22; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000022"),
            },
        ];
        let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
            BTreeMap::new(),
            BTreeMap::new(),
            validators,
        );

        assert!(spec
            .genesis
            .alloc
            .contains_key(&SIMPLEX_VALIDATORS_REGISTRY));
    }

    #[test]
    fn chain_spec_reader_matches_written_validator_registry() {
        let validators = vec![
            ValidatorEntry {
                consensus_pubkey: [0x33; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000033"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            },
        ];
        let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
            BTreeMap::new(),
            BTreeMap::new(),
            validators.clone(),
        );

        let decoded = try_simplex_validators_from_chain_spec(&spec).expect("decode validators");
        assert_eq!(decoded, validators);
    }

    #[test]
    fn validator_registry_encoding_is_independent_of_fee_recipient_registry() {
        let validator_key = [0xaa; 32];
        let fee_recipient = address!("0x00000000000000000000000000000000000000aa");
        let simplex_validators = vec![ValidatorEntry {
            consensus_pubkey: validator_key,
            ethereum_address: address!("0x00000000000000000000000000000000000000bb"),
        }];
        let mut fee_recipients = BTreeMap::new();
        fee_recipients.insert(validator_key, fee_recipient);

        let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
            BTreeMap::new(),
            fee_recipients,
            simplex_validators,
        );

        assert!(spec
            .genesis
            .alloc
            .contains_key(&app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY));
        assert!(spec
            .genesis
            .alloc
            .contains_key(&SIMPLEX_VALIDATORS_REGISTRY));
        assert_ne!(
            app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY,
            SIMPLEX_VALIDATORS_REGISTRY
        );
    }

    #[test]
    fn test_try_build_sahara_chain_spec_with_alloc_rejects_over_cap() {
        let mut alloc = BTreeMap::new();
        let total = sahara_hard_cap_base_units() + U256::from(1u64);
        alloc.insert(
            Address::repeat_byte(0x55),
            GenesisAccount {
                balance: total,
                ..GenesisAccount::default()
            },
        );

        assert_eq!(
            try_build_sahara_chain_spec_with_alloc(alloc),
            Err(NativeTokenError::HardCapExceeded {
                total,
                hard_cap: sahara_hard_cap_base_units(),
            })
        );
    }
}
