## 2026-02-27 Task 1 scaffolding findings
- alloy-* in `vendor/reth/Cargo.toml` `[workspace.dependencies]` are version-based crates.io deps (e.g., `alloy-primitives = { version = "1.5.0", ... }`), not local path deps; no local alloy path wiring is needed from whirlpool crates.
- `app-evm` uses reth path deps only (`../../vendor/reth/crates/...`) for the 7 required reth crates; alloy crates remain transitive through reth crates.
- `revm` does expose primitives via re-exported namespace usage in this repo (`revm::primitives::*` observed under vendor/reth), so `state/Cargo.toml` omits direct `alloy-primitives`.
- `nix develop --command cargo check` first failed due to crates.io timeout downloading `linux-raw-sys v0.12.1`; rerunning the same command succeeded with exit 0 after download completed.
- Non-blocking warnings seen during check/build: deprecated `try_next` in `vendor/commonware/utils/src/channels/tracked.rs` and dead_code field `sink` in `crates/consensus-simplex/src/engine.rs`; no action required for this scaffolding task.


## [2026-02-27T14:45] Task 1: Workspace Scaffolding

**Critical Finding: revm vs reth-revm**
- The plan incorrectly specified `revm = { path = "../../vendor/reth/crates/revm" }`
- Actual vendor crate name: **`reth-revm`** (reth's wrapper with utilities)
- Standalone `revm` crate (core EVM, Database trait) comes from **crates.io version 34**
- Usage pattern:
  - `state` crate: uses `revm = "34"` from crates.io for core Database trait
  - `app-evm` crate: uses `reth-revm = { path = "..." }` for reth-specific execution wiring
- This is the CORRECT pattern, not an error.

**Dependency Resolution**
- alloy-* crates are crates.io deps in reth's workspace (version 0.8), not path deps
- reth-* vendor crates correctly use path deps: `../../vendor/reth/crates/{evm,chainspec,execution-types,...}`
- No async-trait dep added to app crate (uses RPITIT like ConsensusApp)

**Build Status**
- cargo check: PASSED (exit 0, 2 non-blocking warnings)
- cargo build: PASSED (exit 0, "Finished `dev` profile")
- All 3 new crates scaffold correctly with empty stubs

## [2026-02-27T15:38:08Z] Task 3: App Crate Implementation
- Commonware trait import paths: `commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite}`, `commonware_consensus::{Block as VendorBlock, Heightable}`, `commonware_cryptography::{sha256, Committable, Digestible}`.
- ConsensusError mapping: `InvalidBlock` (not `Verification`) in `ApplicationAdapter::verify()` for application-level verification failures.
- RPITIT pattern: `Application` and adapter/consensus bridges use `fn ... -> impl Future<Output = ...> + Send` without `async-trait`; adapter methods wrap delegation in `async move` only where output shape changes (`propose`/`verify`).

## [2026-02-27T15:50:37Z] Task 4: App-EVM Config Implementation
- ConfigureEvm delegation pattern: WhirlpoolEvmConfig is a newtype wrapper over EthEvmConfig and delegates all required ConfigureEvm methods (`block_executor_factory`, `block_assembler`, `evm_env`, `next_evm_env`, `context_for_block`, `context_for_next_block`) directly to `inner`, while preserving associated types via `<EthEvmConfig as ConfigureEvm>::...`.
- ChainSpec builder: `ChainSpecBuilder::default().chain(Chain::from_id(313_371)).genesis(Genesis { gas_limit: 30_000_000, difficulty: U256::ZERO, ..Default::default() }).cancun_activated().build()` yields the expected chain id, genesis gas limit, and Cancun active at timestamp 0; Cancun activation check requires bringing `reth_chainspec::EthereumHardforks` trait into scope in tests.
- Import paths: `alloy_genesis::Genesis`, `alloy_primitives::U256`, `reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder, EthereumHardforks}`, `reth_ethereum_primitives::EthPrimitives`, `reth_evm::{ConfigureEvm, EvmEnvFor, ExecutionCtxFor, NextBlockEnvAttributes}`, `reth_evm_ethereum::EthEvmConfig`, `reth_primitives_traits::{BlockTy, HeaderTy, SealedBlock, SealedHeader}`, `app::ApplicationError`.


## [2026-02-27T16:13:21Z] Task 5 Step 1: Header Conversion Helpers
- Header type used: `reth_primitives_traits::Header` (re-export alias of `alloy_consensus::Header` from `vendor/reth/crates/primitives-traits/src/header/sealed.rs`)
- Required Header fields: non-optional fields on alloy header are `parent_hash`, `ommers_hash`, `beneficiary`, `state_root`, `transactions_root`, `receipts_root`, `logs_bloom`, `difficulty`, `number`, `gas_limit`, `gas_used`, `timestamp`, `mix_hash`, `nonce`, `extra_data`; optional fields are `withdrawals_root`, `base_fee_per_gas`, `blob_gas_used`, `excess_blob_gas`, `parent_beacon_block_root`, `requests_hash` (via extension)
- Hash computation method: `header.hash_slow()` via `alloy_primitives::Sealable`
- SealedHeader constructor: `SealedHeader::new(header, hash)` (also available: `SealedHeader::seal_slow(header)`)

## [2026-02-28 Task 5 Complete] EvmApplication Implementation
- **Resolution**: Manually implemented Application trait with Edit tool to break timeout loop
- **StateProvider trait**: Abstraction added for DB state_root access, allows generic impl
- **Application impl**: genesis() uses computed state_root (NOT EMPTY_ROOT_HASH), propose() MVP empty blocks with parent.timestamp+12, verify() checks state_root mismatch
- **Dependencies resolved**: Added alloy-trie = "0.9" to app-evm/Cargo.toml
- **Visibility fix**: Made EvmBlock::compute_id() public for use in EvmApplication::propose()
- **Tests**: All 4 app-evm tests pass (config + header conversion)
- **Pattern learned**: For ultrabrain tasks with complex type bridging, break into atomic steps OR use manual implementation to avoid timeouts

## [2026-02-27T16:47Z] Task 6: application_integration tests
- `InMemoryStateDb` does not implement `app_evm::executor::StateProvider`; integration tests need a local wrapper (e.g., `TestStateDb`) that delegates `state_root()` to `InMemoryStateDb`.
- `cargo test -p app-evm application_integration` is a name filter and can report 0 tests run; use `cargo test -p app-evm --test application_integration` to execute the integration test target explicitly.
