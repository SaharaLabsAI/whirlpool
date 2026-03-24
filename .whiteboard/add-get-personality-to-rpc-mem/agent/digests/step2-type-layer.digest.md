# Type Layer Digest

- Request key type at RPC boundary: hex string personality ID (`0x...`) decoded to bytes.
- Storage model: `StoredPersonality` with binary fields requiring deterministic RPC encoding.
- Existing helper `decode_hex_field` in `rpc-mem` can be reused for read method input validation.
