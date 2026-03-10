# Alignment Digest — Sub-Intent B

## Intent Summary
Sub-Intent B adds the missing node-side configuration and startup wiring needed for multi-node connectivity. The goal is to let `whirlpool-node` accept explicit network inputs and pass them into the already-complete `p2p-commonware` builder path without changing transport internals or broader consensus behavior.

## Requirements In Scope
- REQ-4: `whirlpool-node` configuration must accept explicit listen addresses, dial peers, and bootstrap peers from CLI/config rather than forcing ephemeral local-only defaults.
- REQ-5: node startup wiring must pass configured listen/dial/bootstrap/validator values into the P2P provider builder.

## Exploration Findings
- `crates/whirlpool-node/src/config.rs` currently exposes only constant defaults: namespace, block interval, bind address, validator seed, and RPC bind address.
- `crates/whirlpool-node/src/main.rs` still hardcodes the actual startup networking values: localhost ephemeral listen/dial addresses, empty bootstrap list, deterministic validator seed, and inline builder wiring.
- `CommonwareNetworkProviderBuilder` already provides the needed setters for listen address, dialable address, bootstrap peers, and validator seeding; no builder API work is needed in this sub-intent.
- No first-party workspace crate currently uses `clap`, but vendored crates in the repository already use Clap 4.x, so a direct dependency in `whirlpool-node` is the clean path.
- The remaining ambiguity is direct dial-peer handling: the explored builder surface clearly covers listen/dialable/bootstrap/validators, but not a direct dial-peer list setter.

## Risks Summary
- Medium: `clap` version compatibility and feature selection because the main workspace has no shared `clap` dependency.
- Medium: bootstrap peer parsing because each input must encode both `PublicKey` and `SocketAddr`.
- Medium: backward compatibility because replacing hardcoded startup values could accidentally change current local-dev behavior.
- High: validator seed to explicit key input transition if Sub-Intent B expands identity handling too aggressively.

## Scope Boundaries
- Included: CLI/config surface design, typed node config structure, startup wiring into `whirlpool-node`, and mapping configured values into existing builder setters.
- Excluded: `p2p-commonware` API redesign, `p2p` trait changes, transport/discovery redesign, consensus relay work, and app business logic changes.
- Excluded requirements: REQ-1, REQ-2, REQ-3, REQ-6, REQ-7, REQ-8.

## Recommended Approach
1. Add derive-based `clap` parsing directly to `crates/whirlpool-node`.
2. Introduce a `NodeConfig` struct in `src/config.rs` that centralizes startup values and preserves current defaults.
3. Parse CLI inputs before Commonware runtime startup and replace inline networking constants in `src/main.rs` with `NodeConfig` fields.
4. Feed configured listen, dialable, bootstrap, and validator values into `CommonwareNetworkProviderBuilder` using the existing builder methods.
5. Keep the pass CLI-first unless a config file format is explicitly approved.

## Approval Gate Readiness
- Sub-Intent B is narrow and implementation-ready for alignment.
- The main open decisions are configuration surface shape, validator identity input format, bootstrap peer string format, and the concrete handling path for direct dial peers.
- No blockers require reopening Sub-Intent A artifacts or changing the existing builder API.

Ready for user approval gate.
