# FLOWS

## Flow 1: Node startup with consensus + RPC lifecycle
1. **Grounded**: Node initializes runtime, network provider, state DB, tx pool, app adapter, and starts consensus engine (`crates/whirlpool-node/src/main.rs::main`).
2. **[PROPOSED]**: Immediately after `engine.start()`, node constructs `EthRpcContext` from cloned handles (`tx_pool`, `state_db`, `height`, chain id).
3. **[PROPOSED]**: Node starts jsonrpsee server task and merges `eth` module.
4. **Grounded**: Node continues waiting via `pending::<()>().await`; both tasks remain alive in same runtime.

### Error path
- RPC bind failure -> process startup should fail fast with explicit log and exit (do not run partially with hidden RPC failure).

### Vendor Runtime Constraints
- Match jsonrpsee macro/server style seen in vendor reth examples (`vendor/reth/examples/node-custom-rpc/src/main.rs`).
- Use jsonrpsee 0.26 family for compatibility with observed vendor pin (`vendor/reth/Cargo.toml`).

## Flow 2: `eth_sendRawTransaction` to tx pool
1. Client submits raw signed tx bytes.
2. **[PROPOSED]** Handler validates minimally (bytes non-empty, hashable; optional decode guard).
3. **Grounded+[PROPOSED]** Handler pushes bytes into shared `InMemoryTxPool::push`.
4. **[PROPOSED]** Handler records tx hash in pending index.
5. Handler returns `B256` tx hash.

### Error path
- Invalid bytes/decode failure -> RPC error with invalid transaction code.
- Lock poisoning on tx pool -> internal RPC error.

## Flow 3: `eth_getBalance` / `eth_getTransactionCount`
1. Client requests account state with optional block id.
2. **[PROPOSED]** Handler resolves supported block ids (`latest`, optionally `pending`) and rejects unsupported historical selectors.
3. **Grounded+[PROPOSED]** Handler reads account from state DB and maps:
   - balance -> `U256`
   - nonce -> numeric RPC type (`U256`/`U64` compatible)
4. Return value.

### Error path
- Unsupported block id/tag -> explicit JSON-RPC invalid params / unsupported selector error.
- State lock read failure -> internal RPC error.

## Flow 4: `eth_estimateGas`
1. Client submits tx request + optional block id.
2. **[PROPOSED]** Handler creates isolated execution context from cloned state snapshot.
3. **[PROPOSED]** Handler performs dry-run for simple transfer path and computes bounded estimate.
4. Return estimated gas value.

### Error path
- Unsupported tx shape for v1 estimator -> explicit method error.
- Execution failure in dry-run -> estimation failed error with reason.

## Flow 5: `eth_getTransactionReceipt` polling
1. Client polls by tx hash.
2. **[PROPOSED]** Handler checks node-local pending/confirmed receipt index.
3. If not confirmed, return `None`.
4. If confirmed, return receipt object containing hash, block/tx position, status, gas usage, logs fields as available.

### Error path
- Hash unknown and never seen -> `None` (not error).
- Partial receipt data unavailable -> return compatible minimal receipt or explicit internal error per documented contract.

## Implementation slices (for downstream planning)
- Slice S1: RPC server bootstrap + `eth_chainId` + `eth_gasPrice`.
- Slice S2: State read methods (`getBalance`, `getTransactionCount`) with block-id support matrix.
- Slice S3: Raw tx ingress (`sendRawTransaction`) + pending index.
- Slice S4: `estimateGas` dry-run strategy.
- Slice S5: Receipt tracking/index and `getTransactionReceipt` polling semantics.
- Slice S6: Alloy integration harness validating send -> receipt -> balance delta.
