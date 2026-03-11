# Requirements

## Scope
- Depth: `module`
- Focus crates: `rpc-eth`
- Intake breadth check: **within threshold**
  - Crates affected: 1 primary + 1 integration boundary (`crates/rpc-eth`, `crates/whirlpool-node`)
  - Boundaries: <=2 (RPC module implementation and node wiring)
  - Domains: 1 primary (JSON-RPC service wiring)
  - Flows: <=3 (request handling, tx pool lookup/submit, chain/state reads)
- Intent split decision: **no split required** (single focused crate replacement)

## Affected Boundaries
- Primary modification target: `crates/rpc-eth`
- Integration/wiring touchpoint: `crates/whirlpool-node` (server construction and adapter injection)
- Read-only dependency surfaces (no edits planned):
  - Vendor reth RPC crates (`reth-rpc`, `reth-rpc-eth-api`, `reth-rpc-builder`, `reth-rpc-eth-types`, `reth-rpc-convert`)
  - Existing compatibility crates used for bridging context (`state-reth`, `app-evm`)

## Requirements
- REQ-1: `rpc-eth` must expose a reth-backed JSON-RPC server by wiring reth `EthApi`/server implementation in place of current hardcoded stub behavior for supported `eth_*` methods.
- REQ-2: Implement `WhirlpoolProvider` adapter that bridges Whirlpool backends (`StateDb`, `BlockStorage`) into the provider trait surface needed by reth RPC execution.
- REQ-3: Implement `WhirlpoolTxPool` adapter that bridges `TxSource` into reth tx-pool expectations for pending transaction reads and transaction submission flow.
- REQ-4: Implement `WhirlpoolNetwork` adapter that satisfies reth network-facing RPC dependencies required by standard `eth_*` methods.
- REQ-5: Blob transaction support must be explicitly excluded; `eth_blobBaseFee` must return an unsupported-method style error/response contract and no EIP-4844 execution path should be introduced.
- REQ-6: `whirlpool-node` must wire the new reth JSON-RPC server path so node startup serves requests through the adapter-backed implementation.
- REQ-7: Integration tests must mirror reth `rpc-builder` test patterns to validate end-to-end wiring for supported standard `eth_*` methods (blob excluded).

## Assumptions
- Provided grounded context is accurate and sufficient for intake without additional crate exploration.
- Existing `state-reth` and `app-evm` compatibility points are adequate to construct adapter types without redesigning core storage or EVM execution architecture.
- Existing Whirlpool backend traits (`StateDb`, `BlockStorage`, `TxSource`) remain authoritative and are adapted rather than replaced.

## Non-Goals
- Implementing Engine API namespaces.
- Implementing admin/debug namespaces.
- Implementing blob transaction handling or full EIP-4844 fee/data-path support.
- Modifying vendor reth crates or redesigning reth RPC internals.
- Producing synthesis/design-phase documents during intake.

## Success Criteria
- Supported standard `eth_*` RPC methods (excluding blob-specific methods) are served via reth-backed adapters instead of hardcoded stubs.
- `eth_blobBaseFee` returns unsupported behavior by design and is covered by tests.
- Integration tests modeled after reth `rpc-builder` patterns pass for the new `rpc-eth` server path.
- `whirlpool-node` starts and serves RPC through the new server wiring with `WhirlpoolProvider`, `WhirlpoolTxPool`, and `WhirlpoolNetwork`.
