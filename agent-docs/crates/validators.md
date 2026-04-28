# validators

## Status
Removed as a package-level compatibility facade.

## Replacement
- Use `validators-reader` (`validators_reader` in Rust) for validator registry reader types/codecs.
- Use `validators-dkg` (`validators_dkg` in Rust) for DKG metadata, activation targets, activation schedules, and canonical extra-data helpers.

## Rationale
The old `validators` crate re-exported registry reader APIs from `evm-precompiles`, which inverted ownership. The split makes registry reading precompile-independent and gives DKG metadata one pure semantic owner.
