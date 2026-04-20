# app-mem

## Purpose
Canonical mem-transaction codec/validation crate for personality-markdown payloads.

## Location
`crates/app/execute/mem/app/`

## Module layout
- `src/lib.rs` — crate facade, module wiring, re-exports
- `src/types.rs` — constants, `SignatureScheme`, `PersonalityMarkdownTx`, `FinalizedPersonalityWrite`
- `src/hash.rs` — SHA-256 hashing helpers (`compute_markdown_hash`, payload hash helper)
- `src/codec.rs` — binary encode/decode and cursor/length helpers
- `src/validation.rs` — transaction semantic validation rules
- `src/tx_constructor.rs` — `PersonalityMarkdownTx::new`
- `src/tx_codec_api.rs` — `PersonalityMarkdownTx::{encode,decode,validate}`
- `src/tx_finalize_api.rs` — `PersonalityMarkdownTx::{finalized_write,tx_hash}`
- `src/api.rs` — crate-level convenience fns (`decode_personality_tx`, `derive_finalized_write`)
- `src/error.rs` — `MemTxError`
- `src/tests.rs` — crate unit tests

## Public API
- Constants:
  - `PERSONALITY_MARKDOWN_TYPE_ID`
  - `SUPPORTED_PERSONALITY_TX_VERSION`
  - `MAX_PERSONALITY_MARKDOWN_BYTES`
  - `MAX_IDENTITY_BYTES`
  - `MAX_SIGNATURE_BYTES`
- Types:
  - `SignatureScheme`
  - `PersonalityMarkdownTx`
  - `FinalizedPersonalityWrite`
  - `MemTxError`
- Functions:
  - `compute_markdown_hash`
  - `decode_personality_tx`
  - `derive_finalized_write`

## Behavioral notes
- Wire format and validation semantics are unchanged by the module split: encoded field order, length-prefix parsing, trailing-bytes checks, and UTF-8/size/hash validation behavior are preserved.
- `tx_hash()` remains hash-of-encoded-payload, while `compute_markdown_hash()` remains hash-of-markdown-bytes.

