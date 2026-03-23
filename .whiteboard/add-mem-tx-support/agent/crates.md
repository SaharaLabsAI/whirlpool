# Crates

## New Crates
- `crates/app-mem`: owns `PersonalityMarkdownTx`-style payload definitions, canonical encoding/decoding, size/hash/UTF-8 validation, transaction classification helpers, and derivation of finalized personality writes from accepted block contents.
- `crates/rpc-mem`: owns experimental mem-facing JSON-RPC methods, starting with `mem_submitPersonality`, plus request validation and a narrow submission service boundary into the shared transaction ingress path.

## Existing Crates To Change
- `crates/app`: keep shared abstractions generic; `TxSource` already supports opaque bytes, so terminology and comments should shift from "raw EIP-2718 bytes" toward generic signed transaction bytes where needed.
- `crates/app-evm`: keep EVM execution isolated, but remove the assumption that every pending or block transaction is EVM-decodable; proposal and verification should cooperate with `app-mem` classification instead of treating non-EVM bytes as malformed EVM.
- `crates/whirlpool-node`: instantiate the personality store, start both `rpc-eth` and `rpc-mem`, and extend finalization wiring so finalized personality writes flush alongside finalized block persistence.
- `crates/state` or `crates/state-memory`: add the prototype personality storage trait and in-memory backend; `BlockStorage` stays focused on finalized blocks and receipts.
- `crates/mempool-mdbx`: likely no semantic change beyond confirming it remains payload-agnostic, since it already stores and drains opaque `Vec<u8>` entries.
- `crates/rpc-eth`: no feature expansion; keep it Ethereum-only and avoid mixing non-EVM request types into the reth-backed surface.

## Grounded Notes
- `Cargo.toml` does not yet include `crates/app-mem` or `crates/rpc-mem`, so both need workspace membership.
- `crates/state/src/block_storage.rs` is block/receipt-specific today, which supports adding a separate personality storage abstraction instead of overloading block storage.
- `crates/state-memory/src/lib.rs` already exposes an in-memory state crate, making it a natural home for a prototype in-memory personality backend if kept separate from block storage concerns.
