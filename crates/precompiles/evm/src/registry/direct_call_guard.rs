use alloy_primitives::Bytes;
use alloy_sol_types::{sol, SolError};
use reth_evm::revm::precompile::{PrecompileOutput, PrecompileResult};

sol! {
    /// Shared framework-level error used when a Whirlpool-owned stateful precompile
    /// is invoked through a non-direct path such as DELEGATECALL or CALLCODE.
    #[derive(Debug, PartialEq, Eq)]
    error NonDirectCall();
}

pub fn non_direct_call_revert_bytes() -> Bytes {
    Bytes::from(NonDirectCall {}.abi_encode())
}

pub fn non_direct_call_revert_result() -> PrecompileResult {
    // `REVERT` does not imply zero gas in general, but this framework-level rejection happens
    // before the precompile executes any opcode-equivalent work or applies its own gas policy.
    // We therefore report zero precompile gas here and let the enclosing EVM machinery account
    // for any call/setup cost outside the precompile itself.
    Ok(PrecompileOutput::new_reverted(
        0,
        non_direct_call_revert_bytes(),
    ))
}
