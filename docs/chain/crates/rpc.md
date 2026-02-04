# `rpc`

**Purpose**: client-facing API surface for submitting txs and querying chain data.

Owns: request/response types, server handlers, auth/limits, subscriptions (if any).

Implements: `core::Rpc` (trait boundary).

Depends on: `core` (traits), `types`, plus read access to `storage` (and optionally `executor` for tx checks).

Not in scope: P2P networking, consensus rules.
