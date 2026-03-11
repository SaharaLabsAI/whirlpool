# Skill Digest

## Grounded
- **Align verdict**: APPROVED (user pre-approved) — review/alignment-digest.md
- **Scope**: rpc-eth crate replacement + whirlpool-node integration — agent/requirements.md
- **REQ count**: 7 (REQ-1..REQ-7) — agent/requirements.md
- **TST count**: 12 (TST-1..TST-12) — agent/tests.md
- **Risks accepted**: 3 (R1:provider surface, R2:blob interleave, R3:type conversion) — scratch/risk-assessment.md
- **No split**: single crate, clear boundaries — requirements.md threshold check
- **Reth RPC crates**: reth-rpc-eth-api (traits), reth-rpc (impl), reth-rpc-builder (wiring), reth-rpc-eth-types (types), reth-rpc-convert (conversion)
- **Provider adapter surface**: ~20 reth storage traits, NoopProvider shows minimum
- **Existing bridges**: state-reth (StateDb→revm::Database, BlockStorage→reth tables), app-evm (reth_evm, TransactionSigned, SealedHeader)
- **Test reference**: vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs (param permutation), utils.rs (NoopProvider test fixtures)

## [PROPOSED]
- (none yet — design phase not started)

## Unknowns
- Exact set of Provider trait methods that need real (non-stub) implementations for core eth_ methods
- Whether CanonStateSubscriptions can be satisfied with a no-op for our use case
