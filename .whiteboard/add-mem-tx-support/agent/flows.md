# Architecture Flows

## Flow 1: Mem RPC submission to shared ingress

```text
Client -> `crates/rpc-mem::mem_submitPersonality`
    -> request shape + size + UTF-8 validation
    -> canonical mem payload encoding + tx hash
    -> memory-ingress adapter over `app::traits::TxSource`
    -> shared opaque-byte mempool queue
```

Decision: keep `crates/rpc-eth` Ethereum-only and start mem submission through a separate `crates/rpc-mem` surface.

## Flow 2: Proposal classification in mixed mempool

```text
`crates/app-evm` / node-owned mixed proposal path
    -> `TxSource::pending()` returns raw bytes
    -> classify each item as EVM | mem | invalid
    -> execute EVM txs with existing semantics
    -> structurally validate mem txs via `crates/app-mem`
    -> include accepted ordered bytes in block body
    -> retain derived mem writes as pending finalization data
```

Decision: preserve a payload-agnostic mempool and move transaction-family decisions to deterministic proposal logic.

## Flow 3: Verification and consensus safety

```text
Received block
    -> decode every tx deterministically
    -> re-execute EVM txs
    -> re-validate mem tx format/hash/limits/signature shape
    -> reconstruct derived mem writes
    -> confirm block commitments still match
```

Decision: keep v1 verification structural-only for mem signatures; Jolt-backed authenticity remains explicitly deferred.

## Flow 4: Finalization-only personality visibility

```text
Consensus finalizes block
    -> finalization sink reads pending mem writes
    -> node-owned in-memory personality store applies writes
    -> `personality_id` latest finalized value wins
    -> block persistence continues through existing finalized-block path
```

Decision: finalized personality state becomes visible only after finalization, matching the current canonicality boundary.

## Flow 5: Dual-server node ownership

```text
`crates/whirlpool-node/src/node.rs`
    -> build shared state DB + shared TxSource + personality store
    -> start `rpc-eth` server for Ethereum methods
    -> start `rpc-mem` server for mem methods
    -> wire both to the same chain and mempool dependencies
```

Decision: `whirlpool-node` owns lifecycle composition; experimental mem concerns stay boxed into `app-mem`, `rpc-mem`, and the finalization-store adapter.
