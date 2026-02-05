# `rpc`

**Purpose**: client-facing API surface for submitting txs and querying chain data.

Owns: request/response types, server handlers, auth/limits, subscriptions (if any).

Implements: `core::Rpc` (trait boundary).

Depends on: `core` (traits), `types`, plus read access to `storage` (and optionally `executor` for tx checks).

Not in scope: P2P networking, consensus rules.

## Transaction submission (shape)

RPC accepts **raw signed transaction bytes only** (Ethereum-style `sendRawTransaction`).

- Input: `Bytes` containing an encoded signed tx (family-specific; for EVM this is the signed envelope).
- Server behavior: decode -> basic validation/rate limits -> forward to `mempool`/`network`.
- Output: a stable transaction identifier (typically `TxHash`).

Unsigned tx submission and server-side signing are out of scope.
