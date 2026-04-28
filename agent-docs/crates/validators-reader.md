# validators-reader

## Purpose
Canonical Whirlpool validator registry reader crate.

## Location
`crates/validators/reader/`

## Owns
- `ValidatorEntry { consensus_pubkey, ethereum_address }`
- `SIMPLEX_VALIDATORS_REGISTRY`
- `encode_validator_registry_storage(entries)`
- `decode_validator_registry_storage(storage)`
- `decode_validator_registry_storage_opt(storage)`
- `ordered_consensus_pubkeys(entries)`
- `encode_ethereum_address_storage_value(address)`
- `ValidatorRegistryError`

## Boundary
This crate owns registry storage representation and Rust reader semantics only. It does not own precompile ABI execution, activation schedules, DKG metadata, or EVM runtime behavior.

## Consumers
- `chainspec` writes genesis validator registry storage.
- `whirlpool-node`, RPC/integration tests, and `app-evm-execution` consume ordered validator entries.
- `evm-precompiles` consumes `ValidatorEntry` for the validators precompile ABI output, but no longer defines registry reader semantics.

## Verification
- `cargo test -p validators-reader`
- Dependency gate: `validators-reader` must not depend on `evm-precompiles` or `validators-dkg`.
