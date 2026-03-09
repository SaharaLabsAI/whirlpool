# Implementation Handoff

## Intent
- Finalize Sub-Intent A for P2P provider completeness only.
- Scope is limited to `REQ-1`, `REQ-2`, and `REQ-3`.
- Use the stable `crates/p2p` abstraction as the compatibility boundary.

## Inputs
- Strategy: `docs/design/p2p-node-connectivity/agent/strategy.md`
- Crate specs: `docs/design/p2p-node-connectivity/agent/crates.md`
- Workspace plan: `docs/design/p2p-node-connectivity/agent/workspace.md`
- Domains: `docs/design/p2p-node-connectivity/agent/domains.md`
- Flows: `docs/design/p2p-node-connectivity/agent/flows.md`
- Tests: `docs/design/p2p-node-connectivity/agent/tests.md`
- Crate contracts:
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/p2p-commonware.md`
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/whirlpool-node.md`

## Implementation Order
1. Update `crates/p2p-commonware/src/receiver.rs`.
   - Dependency: none
   - Required change: store the concrete `p2p::Channel` on `CommonwareReceiver` and emit it from `recv()`.
   - Why first: this is the smallest isolated fix and establishes the channel-preservation contract for `REQ-3`.
2. Update `crates/p2p-commonware/src/provider.rs`.
   - Dependency: step 1
   - Required change: pass `Channel::VOTE`, `Channel::CERTIFICATE`, and `Channel::RESOLVER` into `CommonwareReceiver::new(...)`; apply validator seeding in `CommonwareNetworkProviderBuilder::build(context)`; preserve bootstrap threading into `discovery::Config::local(...)`.
   - Why second: provider construction is the central runtime assembly point for `REQ-1`, `REQ-2`, and receiver call-site updates.
3. Update `crates/p2p-commonware/src/lib.rs`.
   - Dependency: step 1
   - Required change: keep `MultiplexReceiver` aligned with receiver-owned channel tagging and remove any assumption that it must repair channel metadata.
   - Why third: this keeps the crate-level aggregate path consistent with the fixed receiver contract.
4. Review `crates/p2p-commonware/src/sender.rs` and `crates/p2p-commonware/src/traits.rs` for compatibility-only adjustments.
   - Dependency: step 2
   - Required change: none expected beyond import normalization or test updates.
   - Why fourth: these files should only move if the earlier changes require compile-fix alignment.
5. Update `crates/whirlpool-node/src/main.rs`.
   - Dependency: step 2
   - Required change: pass the startup validator set via `initial_validators(...)` and supply bootstrap peers via `bootstrappers(...)` while preserving existing defaults.
   - Why fifth: node wiring depends on the provider-side builder contract being finalized.
6. Implement tests defined in `docs/design/p2p-node-connectivity/agent/tests.md`.
   - Dependency: steps 1-5
   - Required change: add/adjust crate-local tests for validator seeding, bootstrap threading, and channel preservation.
   - Why last: tests should validate the completed implementation shape.

## Dependency Notes
- `crates/whirlpool-node/src/main.rs` must not bypass provider-owned validator seeding by calling `oracle_handle.update_validators(...)` directly.
- `crates/p2p-commonware/src/provider.rs` is the only place where bootstrap inputs become Commonware discovery configuration and where initial validators become oracle state.
- `crates/p2p-commonware/src/receiver.rs` and `crates/p2p-commonware/src/lib.rs` must agree on a single source of truth for channel metadata: the concrete channel assigned at receiver construction.

## Acceptance Checks
- `REQ-1`: provider build seeds validators before the provider is handed off.
- `REQ-2`: startup wiring no longer leaves bootstrap peers empty by construction when bootstrap inputs exist for the pass.
- `REQ-3`: sender -> channel -> receiver path preserves `NetworkMessage.channel` for vote, certificate, and resolver traffic.
- No source changes extend into Sub-Intent B or C.
- No vendor code is touched.

## Downstream Readiness
- After implementation, plan generation can decompose work by file and test contract directly from this handoff.
- Later Sub-Intent B may build on the same builder inputs for CLI/config ergonomics.
- Later Sub-Intent C may assume `NetworkMessage.channel` is trustworthy and no longer needs compensating remap logic.
