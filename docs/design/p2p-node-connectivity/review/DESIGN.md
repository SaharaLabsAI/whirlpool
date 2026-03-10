# Design Review Summary

## Verdict
- PASS for Sub-Intent B finalization.
- Scope remains correctly limited to `REQ-4` and `REQ-5`.
- No blocking design gaps remain; `agent/blockers.md` is already PASS and consistent with finalized artifacts.

## Reviewed Scope
- Primary implementation crate: `crates/whirlpool-node`
- Read-only dependency boundary: `crates/p2p-commonware`
- Verified source anchors:
  - `crates/whirlpool-node/src/config.rs`
  - `crates/whirlpool-node/src/main.rs`
- Explicitly deferred:
  - config-file support
  - multi-validator startup inputs
  - keystore/private-key material inputs
  - peer deduplication
  - any `crates/p2p-commonware` API change

## Source-Grounded Findings
- `crates/whirlpool-node/src/config.rs` currently contains only loose startup constants, so the finalized design correctly elevates this file into the canonical config module.
- `crates/whirlpool-node/src/main.rs` currently hardcodes startup namespace, listen and dialable addresses, bootstrap peers, storage directories, max message size, consensus namespace wiring, and RPC bind parsing, so Sub-Intent B is correctly targeted at centralizing these values behind `NodeConfig`.
- The design preserves a real source-level distinction already present in the codebase:
  - network namespace is currently hardcoded as `b"whirlpool-dev"`
  - consensus namespace is currently sourced from `config::NAMESPACE` as `b"sahara-chain-v0"`
- The review confirms the design does not invent unsupported upstream behavior: dial peers are normalized into Commonware bootstrappers because that is the existing builder contract.

## Decision Summary
- `crates/whirlpool-node/src/config.rs` becomes the canonical home for:
  - `NodeArgs`
  - `NodeConfig`
  - nested config structs
  - default values
  - bootstrap peer parsing
  - storage path derivation
- `crates/whirlpool-node/src/main.rs` parses CLI arguments before runtime creation and replaces hardcoded startup literals with config-owned values.
- `--dial-peer` and `--bootstrap-peer` are two CLI surfaces for one internal model: `Vec<p2p_commonware::Bootstrapper<commonware_cryptography::ed25519::PublicKey>>`.
- The Commonware builder contract stays unchanged and receives configured values through existing setters only.
- Backwards compatibility is explicit: no-flag startup must behave exactly like the current local-dev node.

## Artifact Summary
- Agent-lane finalized outputs:
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/whirlpool-node.md`
  - `docs/design/p2p-node-connectivity/agent/flows.md`
  - `docs/design/p2p-node-connectivity/agent/tests.md`
  - `docs/design/p2p-node-connectivity/agent/handoff.md`
  - `docs/design/p2p-node-connectivity/agent/TASK_GEN_READY.md`
- Supporting design inputs remain:
  - `docs/design/p2p-node-connectivity/agent/strategy.md`
  - `docs/design/p2p-node-connectivity/agent/crates.md`
  - `docs/design/p2p-node-connectivity/agent/domains.md`
  - `docs/design/p2p-node-connectivity/agent/workspace.md`
  - `docs/design/p2p-node-connectivity/agent/blockers.md`
  - `docs/design/p2p-node-connectivity/agent/requirements.md`
- Review-lane outputs:
  - `docs/design/p2p-node-connectivity/review/DESIGN.md`
  - `docs/design/p2p-node-connectivity/review/INDEX.md`

## Review Checks
- Contract completeness: PASS
  - full startup config surface documented
  - full builder wiring contract documented
  - preconditions, postconditions, error handling, and compatibility guarantees documented
- Flow completeness: PASS
  - CLI parsing flow documented
  - startup wiring flow documented
  - bootstrap peer parsing flow documented
  - storage derivation flow documented
- Testability: PASS
  - unit tests cover defaults, parsing, normalization, and path derivation
  - integration tests cover default compatibility and custom startup wiring
- Scope discipline: PASS
  - no Rust source changes made in finalize
  - no `crates/p2p-commonware` redesign proposed
  - no `e2e-state.md` updates involved

## Residual Risks
- Exact conversion error shape may require `TryFrom<NodeArgs>` instead of `From<NodeArgs>` if clap integration does not own peer parsing directly; this is an implementation detail, not a design blocker.
- Wiring `block_interval` into the live consensus startup path may require clarifying which exact timeout field should consume it if implementation wants stronger semantic alignment than the current fixed durations.
- These are implementation-time details only; the design remains task-generation ready.
