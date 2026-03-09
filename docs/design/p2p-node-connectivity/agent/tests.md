# Test Contracts

## Scope
- Sub-Intent A only: `REQ-1`, `REQ-2`, `REQ-3`
- Concrete test targets reference exact implementation files:
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/p2p-commonware/src/receiver.rs`
  - `crates/p2p-commonware/src/sender.rs`
  - `crates/p2p-commonware/src/lib.rs`
  - `crates/p2p-commonware/src/traits.rs`
  - `crates/whirlpool-node/src/main.rs`

## Requirement Traceability
- `REQ-1` -> `TST-REQ1-001`, `TST-REQ1-002`
- `REQ-2` -> `TST-REQ2-001`, `TST-REQ2-002`
- `REQ-3` -> `TST-REQ3-001`, `TST-REQ3-002`, `TST-REQ3-003`

## Test Contracts

### `TST-REQ1-001` Provider build seeds non-empty validator set
- Requirement: `REQ-1`
- Target files:
  - `crates/p2p-commonware/src/provider.rs`
- Test type: unit/integration test in the crate test module for `provider.rs`
- Setup:
  - build a `CommonwareNetworkProviderBuilder` with a deterministic signer
  - pass `initial_validators(epoch=0, vec![pk0, pk1])`
  - use deterministic runtime context
- Assertion:
  - the builder path calls `OracleHandle::update_validators(0, validators)` before returning
  - the resulting oracle peer set contains exactly the deduplicated validator keys
- Failure caught:
  - builder silently discards `initial_validators`
  - seeding is deferred until after provider handoff

### `TST-REQ1-002` Empty validator set skips seeding without failing build
- Requirement: `REQ-1`
- Target files:
  - `crates/p2p-commonware/src/provider.rs`
- Test type: unit test
- Setup:
  - build the provider with `initial_validators(epoch=0, vec![])` or with no validator tuple
- Assertion:
  - `build(context)` succeeds
  - no panic occurs
  - oracle state remains unchanged from the unseeded baseline
- Failure caught:
  - empty validator input causes invalid peer-set construction or unnecessary update calls

### `TST-REQ2-001` Builder threads supplied bootstrappers into discovery config
- Requirement: `REQ-2`
- Target files:
  - `crates/p2p-commonware/src/provider.rs`
- Test type: focused unit/integration test
- Setup:
  - create a builder with a known bootstrapper list and deterministic signer
  - build the provider using deterministic runtime context
- Assertion:
  - the discovery/network startup path uses the exact bootstrapper values supplied to `bootstrappers(...)`
  - provider startup succeeds with those bootstrappers present
- Failure caught:
  - builder overwrites or drops the bootstrapper list before `discovery::Config::local(...)`

### `TST-REQ2-002` Node startup wiring populates builder bootstrappers and validators together
- Requirement: `REQ-2` with `REQ-1` integration coverage
- Target files:
  - `crates/whirlpool-node/src/main.rs`
  - `crates/p2p-commonware/src/provider.rs`
- Test type: startup wiring test or construction-level integration test
- Setup:
  - exercise the node startup builder assembly with placeholder/local bootstrap peers and the startup validator set
- Assertion:
  - `CommonwareNetworkProviderBuilder` receives both `bootstrappers(...)` and `initial_validators(...)` before `.build(...)`
  - runtime defaults for namespace, max message size, and ephemeral listen/dial addresses remain unchanged
- Failure caught:
  - `main.rs` still constructs the provider with an empty bootstrapper list by default
  - node startup bypasses the builder seeding path

### `TST-REQ3-001` Receiver emits configured vote channel
- Requirement: `REQ-3`
- Target files:
  - `crates/p2p-commonware/src/receiver.rs`
  - `crates/p2p-commonware/src/provider.rs`
- Test type: unit test or deterministic receive-path integration test
- Setup:
  - construct `CommonwareReceiver::new(Channel::VOTE, receiver)` with a test receiver that yields one payload
- Assertion:
  - `recv()` returns `NetworkMessage.channel == Channel::VOTE`
  - payload bytes and sender identity are preserved
- Failure caught:
  - `recv()` still hard-codes `Channel(0)`

### `TST-REQ3-002` Receiver emits configured certificate and resolver channels distinctly
- Requirement: `REQ-3`
- Target files:
  - `crates/p2p-commonware/src/receiver.rs`
  - `crates/p2p-commonware/src/provider.rs`
- Test type: unit test
- Setup:
  - create one receiver with `Channel::CERTIFICATE`
  - create one receiver with `Channel::RESOLVER`
  - feed distinct payloads through each
- Assertion:
  - certificate messages retain `Channel::CERTIFICATE`
  - resolver messages retain `Channel::RESOLVER`
  - the two lanes do not collapse to the same channel ID
- Failure caught:
  - per-channel receiver construction ignores the constructor-provided channel

### `TST-REQ3-003` Multiplex receiver forwards already-tagged message without repair logic
- Requirement: `REQ-3`
- Target files:
  - `crates/p2p-commonware/src/lib.rs`
  - `crates/p2p-commonware/src/receiver.rs`
- Test type: unit test for `MultiplexReceiver`
- Setup:
  - construct a `MultiplexReceiver::new_for_test(...)` with vote/certificate/resolver `CommonwareReceiver` instances
  - feed one message on each channel in sequence
- Assertion:
  - each returned `NetworkMessage.channel` matches the originating receiver's configured channel
  - `MultiplexReceiver` does not rewrite all outputs to a single value
- Failure caught:
  - leftover compensation logic masks receiver-level bugs or collapses channel metadata

## Completion Criteria
- Every `REQ-*` in scope maps to at least one concrete `TST-*`.
- Tests remain inside the affected crates; no new end-to-end harness is required in this finalize pass.
- Test assertions verify exact behavior changes, not generic success conditions.
