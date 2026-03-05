## Grounded facts
- Vendor pattern files were identified and read:
  - `vendor/reth/examples/node-custom-rpc/src/main.rs`
  - `vendor/reth/examples/rpc-db/src/myrpc_ext.rs`
  - `vendor/reth/Cargo.toml` for version pin.
- These provide sufficient runtime lifecycle + macro trait patterns for this design.

## [PROPOSED] deltas
- Treat vendor examples as canonical style references for implementation phase input packs.
- No additional vendor mining required for current scope.
