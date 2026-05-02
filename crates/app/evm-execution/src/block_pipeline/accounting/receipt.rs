use alloy_consensus::TxReceipt;

use crate::error::EvmAppError;
use crate::invariants::block_pipeline::accounting::cumulative_gas_delta;

pub fn gas_deltas_and_used<R>(receipts: &[R]) -> Result<(Vec<u64>, u64), EvmAppError>
where
    R: TxReceipt,
{
    let mut previous = 0_u64;
    let mut deltas = Vec::with_capacity(receipts.len());

    for receipt in receipts {
        let cumulative = receipt.cumulative_gas_used();
        let delta = cumulative_gas_delta(previous, cumulative).ok_or_else(|| {
            EvmAppError::InvalidBlock(format!(
                "receipt cumulative gas must be nondecreasing: previous={previous}, current={cumulative}"
            ))
        })?;
        deltas.push(delta);
        previous = cumulative;
    }

    Ok((deltas, previous))
}
