# `network`

**Purpose**: authenticated peer connectivity + message transport.

Owns: discovery, peer set mgmt, gossip + request/response plumbing, rate limits/DoS controls.

Implements: `core::Network` (trait boundary).

Inputs/outputs: trait methods exchange `types` (blocks/txs/votes) and network events.

Depends on: `core` (traits), `types`.
