use crate::RegisteredPrecompile;

pub trait WhirlpoolStatefulPrecompile {
    fn register() -> RegisteredPrecompile;
}
