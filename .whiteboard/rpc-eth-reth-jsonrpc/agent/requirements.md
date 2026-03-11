# Requirements

## Scope
- Depth: `module`
- Focus crates: `rpc-eth` (primary), `whirlpool-node` (integration boundary)
- Phase: synthesize (design artifacts only)

## Affected Boundaries
- Primary redesign target: `crates/rpc-eth`
- Integration touchpoint: `crates/whirlpool-node` RPC startup wiring
- Read-only references:
  - `vendor/reth/**`
  - existing Whirlpool crates used as adapter substrates (`state`, `state-reth`, `app`, `app-evm`)

## Requirements

- REQ-1: `rpc-eth` must be designed to expose reth-backed Ethereum JSON-RPC by composing reth `EthApi` modules through `RpcModuleBuilder`, replacing the current hand-rolled server/handler path.

- REQ-2: A `WhirlpoolProvider` adapter must be designed to bridge Whirlpool storage backends into reth provider bounds required for RPC build (`FullRpcProvider`, `CanonStateSubscriptions`, `PersistedBlockSubscriptions`, `AccountReader`, `ChangeSetReader`).

- REQ-3: A `WhirlpoolTxPool` adapter must be designed to bridge `app::traits::TxSource` into `reth_transaction_pool::TransactionPool` for raw transaction submission and pending transaction exposure.

- REQ-4: A `WhirlpoolNetwork` adapter must be designed to satisfy `reth_network_api::NetworkInfo + reth_network_api::Peers` with deterministic minimal behavior suitable for RPC module wiring.

- REQ-5: Blob support must be explicitly excluded: `eth_blobBaseFee` behavior is contracted as unsupported, and no EIP-4844 execution/data path is introduced in this integration.

- REQ-6: Type-3 blob transaction submission must be rejected at `WhirlpoolTxPool` boundary with explicit unsupported-style error behavior.

- REQ-7: `whirlpool-node` must be designed to call the new adapter-backed `rpc-eth` server composition path while preserving node lifecycle ownership and startup order.

## Assumptions
- Vendored reth contracts remain stable for this integration cycle.
- `state-reth::RethStateDb` remains the primary substrate for state/block adapter reads.
- `TxSource` remains authoritative for transaction ingress and pending transaction visibility.

## Non-Goals
- Implementing runtime code changes in `crates/*` during this synth pass.
- Modifying vendored reth crates.
- Adding engine/admin/debug namespace functionality.
- Adding blob/EIP-4844 support.

## Success Criteria
- Design docs define a complete, internally consistent adapter architecture (`WhirlpoolProvider`, `WhirlpoolTxPool`, `WhirlpoolNetwork`).
- Builder bounds and trait obligations are explicitly represented in crate/domain docs.
- Blob exclusion contract is explicit and testable (`eth_blobBaseFee` unsupported + type-3 rejection).
- Requirements map cleanly to test contracts TST-1..TST-12.
