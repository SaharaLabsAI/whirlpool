# Blockers

PASS

- No blocking design gaps remain for Sub-Intent B after synthesis.
- `crates/p2p-commonware` already exposes all required builder inputs for REQ-4 and REQ-5, so no upstream API work is needed.
- The dial-peer ambiguity is resolved in-scope: for `whirlpool-node`, dial peers are modeled as Commonware bootstrappers and parsed as `Vec<Bootstrapper<ed25519::PublicKey>>`.
- The namespace mismatch is resolved in-scope by carrying separate config values for network namespace and consensus namespace.
- The storage-path question is resolved in-scope by using a single `--data-dir` root with fixed `runtime`, `state`, and `mempool` derived subpaths.
- Remaining work is implementation in `crates/whirlpool-node`, not further design investigation.

## Non-blocking follow-ups
- A later pass may add config-file support if operators need persisted startup profiles.
- A later pass may add explicit private key or keystore inputs if deterministic seed-only identity becomes too limiting.
- A later pass may revisit peer deduplication and richer peer-source precedence if the CLI surface grows.
