use alloy_consensus::{SignableTransaction, Transaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, TxKind, U256};
use app::EvmBlock;
use evm_precompiles::{
    advance_epoch_calldata, epoch_system_tx_sender, is_advance_epoch_calldata, next_epoch_block_slot,
    EPOCH_PRECOMPILE_ADDRESS,
    EPOCH_SYSTEM_TX_GAS_LIMIT, EPOCH_SYSTEM_TX_PRIVATE_KEY,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::crypto::secp256k1::sign_message;

use crate::{error::EvmAppError, traits::StateProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochBoundaryState {
    pub next_epoch_block: u64,
    pub system_sender_nonce: u64,
}

pub fn load_epoch_boundary_state<DB>(db: &DB) -> Result<EpochBoundaryState, EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let next_epoch_raw = db
        .get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
        .map_err(Into::into)?;
    let next_epoch_block = u64::try_from(next_epoch_raw).map_err(|_| {
        EvmAppError::InvalidBlock("epoch nextEpochBlock storage does not fit into u64".into())
    })?;

    let sender_info = db
        .get_account(epoch_system_tx_sender())
        .map_err(Into::into)?
        .unwrap_or_default();

    Ok(EpochBoundaryState {
        next_epoch_block,
        system_sender_nonce: sender_info.nonce,
    })
}

pub fn boundary_required_for_height(state: EpochBoundaryState, block_height: u64) -> bool {
    block_height == state.next_epoch_block
}

pub fn build_epoch_boundary_tx(
    chain_id: u64,
    nonce: u64,
    base_fee_per_gas: u64,
) -> Result<Vec<u8>, EvmAppError> {
    let tx = TxLegacy {
        chain_id: Some(chain_id),
        nonce,
        gas_price: base_fee_per_gas as u128,
        gas_limit: EPOCH_SYSTEM_TX_GAS_LIMIT,
        to: TxKind::Call(EPOCH_PRECOMPILE_ADDRESS),
        value: U256::ZERO,
        input: advance_epoch_calldata(),
    };
    let signature = sign_message(EPOCH_SYSTEM_TX_PRIVATE_KEY, tx.signature_hash())
        .map_err(|err| EvmAppError::Execution(format!("failed to sign epoch boundary tx: {err}")))?;
    let signed: TransactionSigned = tx.into_signed(signature).into();
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    Ok(encoded)
}

pub fn tx_is_reserved_epoch_namespace(tx: &TransactionSigned, signer: Address) -> bool {
    if signer != epoch_system_tx_sender() {
        return false;
    }
    if tx.kind() != TxKind::Call(EPOCH_PRECOMPILE_ADDRESS) {
        return false;
    }
    if tx.value() != U256::ZERO {
        return false;
    }
    is_advance_epoch_calldata(tx.input())
}

pub fn validate_epoch_boundary_raw_transactions(
    raw_txs: &[Vec<u8>],
    boundary_required: bool,
    expected_raw_boundary_tx: &[u8],
) -> Result<(), EvmAppError> {
    if boundary_required {
        let first = raw_txs.first().ok_or_else(|| {
            EvmAppError::InvalidBlock("missing required epoch boundary system transaction".into())
        })?;
        if first.as_slice() != expected_raw_boundary_tx {
            return Err(EvmAppError::InvalidBlock(
                "malformed or non-canonical epoch boundary system transaction".into(),
            ));
        }
    }

    for (index, raw_tx) in raw_txs.iter().enumerate() {
        let recovered = match super::decode_evm_transaction(raw_tx) {
            Ok(tx) => tx,
            Err(_) => continue,
        };

        if !tx_is_reserved_epoch_namespace(&recovered, recovered.signer()) {
            continue;
        }

        if !boundary_required {
            return Err(EvmAppError::InvalidBlock(format!(
                "unexpected epoch boundary system transaction on non-boundary block at index {index}"
            )));
        }

        if index != 0 {
            return Err(EvmAppError::InvalidBlock(format!(
                "epoch boundary system transaction must appear at index 0, found at index {index}"
            )));
        }

        if raw_tx.as_slice() != expected_raw_boundary_tx {
            return Err(EvmAppError::InvalidBlock(
                "epoch boundary system transaction does not match canonical bytes".into(),
            ));
        }
    }

    Ok(())
}

pub fn reserved_epoch_namespace_in_raw_tx(raw_tx: &[u8]) -> bool {
    let recovered = match super::decode_evm_transaction(raw_tx) {
        Ok(tx) => tx,
        Err(_) => return false,
    };
    tx_is_reserved_epoch_namespace(&recovered, recovered.signer())
}

pub fn validate_boundary_for_block<DB>(
    db: &DB,
    block: &EvmBlock,
    raw_txs: &[Vec<u8>],
    chain_id: u64,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let state = load_epoch_boundary_state(db)?;
    let boundary_required = boundary_required_for_height(state, block.height);
    let expected_raw =
        build_epoch_boundary_tx(chain_id, state.system_sender_nonce, block.base_fee_per_gas)?;
    validate_epoch_boundary_raw_transactions(raw_txs, boundary_required, &expected_raw)
}
