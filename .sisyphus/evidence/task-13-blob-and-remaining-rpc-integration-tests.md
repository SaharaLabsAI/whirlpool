# Task 13 Evidence

## Scope
Added the remaining RPC integration coverage in `testing/integration-tests/tests/rpc_evm_integration.rs` for TST-8 through TST-11 without modifying `crates/rpc-eth/**` or `vendor/**`.

## Changes made
- Replaced the previous no-op test tx source with a `RecordingTxSource` backed by `Mutex<Vec<Vec<u8>>>` so tests can assert which raw transactions reached the RPC tx source.
- Kept `start_test_rpc()` for the existing tests and added `start_test_rpc_with_tx_source()` to return the server plus the shared recording tx source for new assertions.
- Added helpers for:
  - JSON-RPC POST requests (`post_json`)
  - RPC URL formatting (`rpc_url`)
  - shared reqwest client reuse (`test_client`)
  - serializing RPC tests through a global mutex (`rpc_test_lock`, `lock_rpc_tests`)
  - hex encoding raw transactions (`raw_tx_hex`)
  - constructing signed legacy transaction bytes (`signed_legacy_tx_bytes`)
- Switched the RPC tests to `#[tokio::test(flavor = "current_thread")]` so the test runtime does not over-allocate threads while reth also builds its internal pools.

## New tests
- `tst8_get_block_by_number`
  - Calls `eth_getBlockByNumber` using raw JSON-RPC.
  - Accepts either `null`, a block object, or an RPC error for `latest` on an empty DB.
  - If a block object is returned, asserts block number `0x0`.
- `tst9_send_raw_transaction_acceptance_and_blob_rejection`
  - Sends a signed legacy transaction via `eth_sendRawTransaction`.
  - Verifies the returned hash equals the keccak of the raw bytes.
  - Verifies the legacy raw bytes were recorded by `RecordingTxSource`.
  - Sends a minimal blob-typed raw transaction and asserts the RPC response returns an error mentioning blob/4844/unsupported or decode failure.
  - Verifies the blob transaction was not recorded by the tx source.
- `tst10_blob_base_fee_behavior`
  - Calls `eth_blobBaseFee` via raw JSON-RPC.
  - Accepts either an RPC error or a hex result, ensuring the response is explicit rather than silently malformed.
- `tst11_request_shape_permutations`
  - Verifies an unknown `eth_*` method returns an error.
  - Verifies `eth_gasPrice` returns either an RPC error or a hex string.
  - Verifies `eth_accounts` exposes no unlocked accounts in this test setup (`null` or empty array).

## Validation commands
Required commands after the code change:

```bash
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo build -p integration-tests"
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo test -p integration-tests"
```

## Validation results
- `nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo build -p integration-tests"` passed.
- `nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo test -p integration-tests --test rpc_evm_integration"` passed.
- `nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo test -p integration-tests"` passed.

## Notes
- Existing tests `tst4_server_returns_chain_id`, `tst5_latest_block_number`, `tst6_balance_query_returns_zero_for_empty_db`, and `tst7_eth_syncing_returns_false` were preserved.
- The new blob-path assertion is integration-level and intentionally checks the observable RPC contract: blob transactions are rejected and never forwarded into the recording tx source.
