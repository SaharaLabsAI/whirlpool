# Blockers

## Status: PASS (No hard blockers)

No hard blocker prevents synth design completion for reth-backed `rpc-eth` integration.

## Verified Enablers
1. Required reth crates and trait surfaces are present under local vendored paths.
2. `state-reth::RethStateDb` already bridges Whirlpool state/block storage primitives.
3. `RpcModuleBuilder` contracts and test wiring patterns are discoverable and stable in vendor source.
4. `whirlpool-node` has a single, controlled RPC startup integration point.

## Open Risks (Non-blocking)

### 1) Adapter trait breadth (provider + pool)
- Risk: provider/pool trait surfaces are broad, and incorrect stubs can produce subtle runtime regressions.
- Mitigation: explicit real-vs-stub matrix in `crates.md` and contract tests in `tests.md`.

### 2) Network trait contract completeness
- Risk: implementing only `NetworkInfo` is insufficient because builder bounds also require `Peers`.
- Mitigation: `WhirlpoolNetwork` contract explicitly includes `NetworkInfo + Peers` with deterministic empty peer semantics.

### 3) Blob API surfacing by upstream `EthApi`
- Risk: upstream exposes `eth_blobBaseFee`; without explicit contract, behavior may be ambiguous.
- Mitigation: normative unsupported contract (REQ-5, TST-12), plus tx-pool boundary rejection for type-3 transactions.

### 4) Legacy-to-reth server migration coupling
- Risk: replacing custom context/handler flow can break startup if boundary assumptions drift.
- Mitigation: preserve a single `start_rpc_server` entrypoint and validate node-level wiring contract (TST-12).

## Explicit Non-Blockers
- Increased compile time from additional reth dependencies is expected but not a design blocker.
- Vendored path dependency usage is compatible with synth design and does not require vendor edits.
- No new workspace members or cross-workspace restructuring is required.

## Blocker Gate Verdict
PASS: proceed with synth docs completion and traceability validation.
