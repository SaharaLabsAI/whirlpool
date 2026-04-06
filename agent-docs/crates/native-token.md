# native-token

## Purpose
Owns the canonical Sahara native-token supply invariant.

## Public Surface
- `SAHARA_DECIMALS`: ETH-style decimal count (`18`).
- `SAHARA_HARD_CAP_TOKENS`: whole-token cap (`10_000_000_000`).
- `sahara_hard_cap_base_units() -> U256`: hard cap in base units.
- `total_allocated_supply(&BTreeMap<Address, GenesisAccount>) -> Result<U256, NativeTokenError>`: sums genesis balances.
- `validate_genesis_alloc(&BTreeMap<Address, GenesisAccount>) -> Result<U256, NativeTokenError>`: rejects supply overflow or over-cap allocs.

## Error Model
- `NativeTokenError::SupplyOverflow`: `U256` addition overflow while summing balances.
- `NativeTokenError::HardCapExceeded { total, hard_cap }`: genesis alloc exceeds the protocol cap.

## Consumers
- `app-evm`: validates genesis allocs during Sahara `ChainSpec` construction.
- `whirlpool-node`: re-validates externally supplied `ChainSpec` allocs before startup.
- `integration-tests`: imports the hard-cap helper for exact-cap and over-cap startup tests.

## Status
Active. This crate is the single source of truth for native-token supply limits.
