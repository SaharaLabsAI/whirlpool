use app_evm::EpochBoundaryHook;
use reth_chainspec::ChainSpec;

pub fn declared_epoch_boundary_hook(_chain_spec: &ChainSpec) -> EpochBoundaryHook {
    EpochBoundaryHook::PrecompileSemanticsV1
}
