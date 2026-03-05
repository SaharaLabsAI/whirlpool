# Shared Prior Constraints

## User and orchestration constraints (verbatim)
- "Must use `jsonrpsee` (not axum/hyper/warp)."
- "Must not implement code or modify source files."
- "Must not run `cargo build/test`."
- "Must not create files outside `docs_root` and `scratch_root`."
- "Do NOT make architectural decisions that contradict the existing 3-layer pattern (consensus traits -> adapter -> node)."
- "Required RPC methods to design: `eth_chainId`, `eth_getBalance`, `eth_getTransactionCount`, `eth_estimateGas`, `eth_gasPrice`, `eth_sendRawTransaction`, `eth_getTransactionReceipt`."

## Scope parameters
- `workspace_root=/home/dev/sahara/web3/agent/playground/whirlpool`
- `docs_root=/home/dev/sahara/web3/agent/playground/whirlpool/.design-scratch/e2e/eth-sendrpc-20260305-1445/docs`
- `scratch_root=/home/dev/sahara/web3/agent/playground/whirlpool/.design-scratch/e2e/eth-sendrpc-20260305-1445/scratch`
- `depth=module`
- `focus_crates=whirlpool-node,app`
