# Shared Crate Index Digest

- `rpc-mem` is the change center for method registration and RPC schemas.
- `state` defines `StoredPersonality` and `PersonalityStorage::get_latest` required by read endpoint.
- `state-memory` already provides in-memory read implementation for latest personality.
- `whirlpool-node` must supply rpc-mem with both write ingress and read storage adapters.
