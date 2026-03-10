# Risk Assessment — Sub-Intent B

## Scope
- Focus: REQ-4 and REQ-5 only.
- Crate emphasis: `crates/whirlpool-node` with read-only consumption of existing `p2p-commonware` builder setters.

## Identified Risks

| ID | Risk | Severity | Mitigation |
|----|------|----------|------------|
| B-1 | Adding `clap` may create version drift or feature mismatches because the first-party workspace has no shared `clap` dependency and uses resolver `2`, while vendored crates in the repository already reference Clap 4.x. | Medium | Add `clap` directly in `crates/whirlpool-node` instead of assuming `workspace = true`; prefer a 4.5.x version close to vendored usage; keep features minimal (`derive`, and only `env` if explicitly needed). |
| B-2 | Bootstrap peer parsing is compound data: each entry must produce both an Ed25519 `PublicKey` and a `SocketAddr`, so ambiguous or poorly specified input formats could create invalid startup state or hard-to-debug parse errors. | Medium | Define one canonical CLI format such as `PUBKEY@HOST:PORT`; implement a dedicated parser type in `config.rs`; fail fast with precise parse errors before runtime startup. |
| B-3 | Replacing hardcoded defaults with CLI/config wiring could accidentally break current local development behavior, especially the existing ephemeral localhost listen/dial setup and empty bootstrap list. | Medium | Preserve current constants as defaults in `NodeConfig`; require explicit user input to deviate from `127.0.0.1:0`, empty peer lists, and `127.0.0.1:8545`; treat backward-compatible defaults as a hard acceptance criterion. |
| B-4 | Moving from `VALIDATOR_SEED` toward a broader keypair input model can blur scope and create security/UX uncertainty if the design half-switches between deterministic dev seeds and explicit private keys. | High | Keep deterministic seed input as the default and baseline-compatible path for Sub-Intent B; if explicit private key support is needed, model it as an additional clearly separated option rather than replacing seed input in the same pass. |

## Summary
- High: 1
- Medium: 3
- Low: 0
- Blockers identified: 0

## Recommended Guardrails
- Keep Sub-Intent B CLI-first unless a config file requirement is explicitly approved.
- Separate parsing concerns from startup wiring by converting CLI args into a typed `NodeConfig` before runtime setup begins.
- Do not expand scope into transport, relay, or p2p trait redesign while resolving config risks.
