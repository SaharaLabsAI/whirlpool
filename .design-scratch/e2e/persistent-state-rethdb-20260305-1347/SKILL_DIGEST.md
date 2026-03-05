# SKILL_DIGEST

## Grounded
- **StateDb trait** (state/src/traits.rs): 11 infallible methods — new, with_genesis, state_root, commit, get_account, get_code_by_hash, get_storage, get_block_hash, insert_account, insert_block_hash. **Grounded** (source: state/src/traits.rs)
- **InMemoryStateDb** (state-memory/src/db.rs): HashMap-backed, implements StateDb + revm::Database + revm::DatabaseRef. ~643 lines with tests. **Grounded** (source: state-memory/src/db.rs)
- **whirlpool-node wiring**: TestStateDb wraps InMemoryStateDb, Arc<RwLock<>> passed to EvmApplication + RPC. **Grounded** (source: whirlpool-node/src/main.rs)
- **Consumers**: app-evm (generic over S: StateDb), rpc-eth (generic over S: StateDb). All tests use InMemoryStateDb. **Grounded** (source: grep across crates/)
- **Vendor stack**: reth-db, reth-db-api, reth-db-common, reth-db-models, reth-provider, reth-storage-api, reth-codecs, libmdbx-rs all vendored. **Grounded** (source: vendor/reth/)
- **StateError**: single Internal(String) variant, implements revm DBErrorMarker. **Grounded** (source: state/src/error.rs)

## [PROPOSED]
- New crate `state-reth` implementing StateDb backed by reth-db/MDBX
- Wire into whirlpool-node replacing TestStateDb
- User chose: reth-db backend, full scope (crate + node wiring)

## Unknowns
- Which specific reth-db APIs to use (raw MDBX vs reth-provider abstraction)
- State trie calculation approach (reth-trie or custom)
- Account/storage encoding format (reth-codecs or custom)
