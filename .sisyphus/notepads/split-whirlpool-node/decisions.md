# Decisions

## [2026-02-28T02:41:07Z] Key Decisions from Planning
- Binary names: whirlpool-node (EVM), whirlpool-node-simple (non-EVM)
- Shared bootstrap code: DUPLICATE in each main.rs (not extracted)
- whirlpool-node-simple depends on whirlpool-node as library
- EVM deps made non-optional is safe (lib.rs doesn't expose them)
