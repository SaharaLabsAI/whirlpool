use alloy_primitives::{Address, B256};
use reth_primitives_traits::crypto::secp256k1::{recover_signer, sign_message};
use std::sync::OnceLock;

use crate::RegisteredPrecompile;

use crate::epoch::{r#impl, EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_PRIVATE_KEY};

pub fn register() -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful("whirlpool_epoch", EPOCH_PRECOMPILE_ADDRESS, r#impl::execute)
}

pub fn epoch_system_tx_sender() -> Address {
    static SENDER: OnceLock<Address> = OnceLock::new();
    *SENDER.get_or_init(|| {
        let hash = B256::ZERO;
        let sig = sign_message(EPOCH_SYSTEM_TX_PRIVATE_KEY, hash)
            .expect("epoch system private key must be valid");
        recover_signer(&sig, hash).expect("epoch system signature must recover")
    })
}
