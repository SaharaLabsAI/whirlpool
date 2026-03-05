# Shared Librarian

## Grounded facts
- alloy provider method names and signatures align with eth namespace methods:
  - `eth_chainId`
  - `eth_getBalance`
  - `eth_getTransactionCount`
  - `eth_estimateGas`
  - `eth_gasPrice`
  - `eth_sendRawTransaction`
  - `eth_getTransactionReceipt`
  Evidence: delegated librarian session `ses_3433cbed5ffeo4oQhq0696q3c4` summary.
- Expected response families:
  - chain id -> U64-compatible
  - balance / gas / nonce -> U256/U64-compatible numerics
  - send raw tx -> B256 hash
  - receipt -> optional receipt object
- jsonrpsee 0.26 usage pattern confirmed via vendor examples and librarian findings.

## [PROPOSED] deltas
- Model RPC method signatures directly with alloy primitive/request types to minimize conversion logic.
- Keep block-id optional args supported for compatibility, even if first iteration resolves only `latest` and `pending` semantics.

## UNKNOWNs
- Exact chosen receipt struct path (crate/type) in this workspace will be selected during implementation planning; design mandates shape compatibility with alloy polling logic.
