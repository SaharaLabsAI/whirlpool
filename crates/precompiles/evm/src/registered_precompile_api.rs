use alloy_primitives::Address;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileId, PrecompileResult};

use crate::RegisteredPrecompile;

impl RegisteredPrecompile {
    /// Registers a Whirlpool-owned stateful precompile using the safe default path.
    ///
    /// Precompiles registered here are direct-call-only: the final hop into the
    /// precompile must have `target_address == bytecode_address`, which allows
    /// ordinary `CALL` and `STATICCALL` while rejecting delegate-style execution.
    pub fn new_stateful<F>(name: &'static str, address: Address, handler: F) -> Self
    where
        F: Fn(PrecompileInput<'_>) -> PrecompileResult + Send + Sync + 'static,
    {
        Self {
            address,
            precompile: reth_evm::precompiles::DynPrecompile::new_stateful(
                PrecompileId::custom(name),
                move |input| {
                    if !input.is_direct_call() {
                        // This guard rejects delegate-style entry before the target precompile's
                        // business logic begins. Returning a reverted output with `gas_used = 0`
                        // keeps the precompile-local charge at zero because the handler never ran;
                        // surrounding EVM call overhead is still accounted for by the caller frame.
                        return crate::non_direct_call_revert_result();
                    }
                    handler(input)
                },
            ),
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn precompile(&self) -> reth_evm::precompiles::DynPrecompile {
        self.precompile.clone()
    }
}
