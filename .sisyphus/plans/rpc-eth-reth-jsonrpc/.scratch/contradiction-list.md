# Contradiction List

## Canonical Resolutions
1. `agent/domains.md` preserves a positional `start_rpc_server(...)` signature, but `agent/crate-contracts/rpc-eth.md` and `agent/handoff.md` require `RpcConfig`. The plan uses `RpcConfig` as canonical.
2. `agent/strategy.md` allows either zero blob base fee or unsupported handling, but REQ-5 and TST-10 explicitly require unsupported behavior. The plan uses unsupported behavior plus Type-3 rejection.
3. `agent/handoff.md` lists conversion work after server wiring, while flow definitions require block conversion support before startup wiring is complete. The plan keeps provider-first ordering, then pool/network, then `convert.rs`, then `server.rs`, so compilation order remains valid.

## Verdict
PASS - contradictions resolved in favor of the more specific requirement and test contracts.
