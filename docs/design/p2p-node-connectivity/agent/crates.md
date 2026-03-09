# Crate Change Specifications

## crates/p2p
- No code changes in this synthesize pass.
- Preserve the existing stable contract in `crates/p2p/src/traits.rs` and `crates/p2p/src/types.rs`:
  - `NetworkProvider::start()` returns sender/receiver handles.
  - `NetworkMessage` remains the carrier for `channel`, `data`, and `sender`.
  - `VOTE`, `CERTIFICATE`, and `RESOLVER` remain the canonical channel constants consumed downstream.
- This crate is the compatibility anchor for the provider completeness fixes.

## crates/p2p-commonware

### Files in Scope
- `crates/p2p-commonware/src/provider.rs`
- `crates/p2p-commonware/src/receiver.rs`
- `crates/p2p-commonware/src/sender.rs`
- `crates/p2p-commonware/src/lib.rs`
- `crates/p2p-commonware/src/traits.rs`

### `src/provider.rs`

#### `CommonwareNetworkProviderBuilder`
- Keep the existing fields: `signer`, `namespace`, `listen_addr`, `dialable_addr`, `bootstrappers`, `max_message_size`, `initial_validators`.
- Ensure builder setters exist or are added for:
  - bootstrap peers (`bootstrappers`)
  - initial validators (`initial_validators`)
- Keep builder data in Commonware-native types so `build(context)` does not need late conversion at multiple call sites.

#### `CommonwareNetworkProviderBuilder::build(context)`
- Change this function from passive field storage to active runtime initialization.
- Exact responsibilities:
  1. Build the Commonware-backed provider/runtime with the configured namespace, listen/dial addresses, max message size, and bootstrap peers.
  2. Return `(CommonwareNetworkProvider, OracleHandle)` as today.
  3. If `initial_validators` is non-empty, call `oracle_handle.update_validators(initial_validators.clone())` before returning.
- This is the canonical validator seeding point because it has both the fully constructed oracle handle and the builder-owned validator set.
- The update must happen once per provider build, not lazily on first `start()` call, so discovery/admission state is primed before network traffic begins.

#### Provider receiver construction in `provider.rs`
- Any code path that creates `CommonwareReceiver` instances must pass the real `Channel` value into the receiver constructor.
- If `provider.rs` currently creates the aggregate sender/receiver pair, it must instantiate receivers with `VOTE`, `CERTIFICATE`, and `RESOLVER` (or the matching runtime channel value) instead of relying on receiver-local defaults.
- Use canonical imports from `crate::traits::...` where transport traits are referenced.

### `src/receiver.rs`

#### `CommonwareReceiver` struct
- Add/store a `channel: Channel` field alongside the wrapped Commonware receiver handle.
- Keep sender identity extraction unchanged.

#### `CommonwareReceiver::recv()`
- Replace the hard-coded `Channel(0)` in the emitted `NetworkMessage` with the stored `self.channel`.
- Preserve existing behavior for payload bytes and authenticated sender identity.
- This is the only behavioral change required for REQ-3.

#### `CommonwareReceiver::new(...)`
- Update constructor signature to accept the concrete `Channel` for that receiver instance.
- Constructor callers must now supply the channel explicitly; no fallback/default channel should remain.

### `src/sender.rs`
- No primary bug fix required here.
- Confirm send-path channel routing continues to use the caller-supplied `Channel` and maps recipients correctly.
- If imports are touched while adjusting constructor signatures, normalize them to the `crate::traits::...` pattern.

### `src/lib.rs`
- If `MultiplexReceiver` or helper constructors allocate per-channel receivers here, update those constructors to pass through the originating `Channel` to `CommonwareReceiver`.
- Preserve the existing round-robin multiplexing behavior; no fairness or polling redesign is needed.
- Keep exported module surface stable.

### `src/traits.rs`
- No interface redesign.
- Use as the canonical local import site for Commonware transport traits referenced by sibling modules.
- If minor re-exports are needed to support the `crate::traits::` import convention consistently, keep them additive and local to this crate.

## crates/whirlpool-node

### File in Scope
- `crates/whirlpool-node/src/main.rs`

### `main.rs` startup wiring
- Update the P2P builder assembly site so it supplies:
  - the already-derived validator identity set into `CommonwareNetworkProviderBuilder::initial_validators(...)`
  - bootstrap peer list into `CommonwareNetworkProviderBuilder::bootstrappers(...)`
- Preserve the current namespace and max message size defaults.
- Preserve current ephemeral `127.0.0.1:0` defaults for this pass when explicit network config is absent.
- Continue keeping the returned `oracle_handle` alive for the runtime lifetime; no new ownership model is required.

### Validator seeding integration detail
- The validator set created during startup is already the canonical source of truth for consensus membership at boot.
- Convert/map that validator set once in `main.rs` into the Commonware/public-key form expected by the builder.
- Do not call `oracle_handle.update_validators(...)` directly in `main.rs`; that responsibility remains centralized in `provider.rs` so all callers benefit consistently.

### Bootstrap integration detail
- For this pass, bootstrap peers come from the node wiring layer, even if currently sourced from placeholder/local values.
- The important change is that `main.rs` stops leaving the builder bootstrap list empty by construction.
- CLI/config ergonomics for user-specified bootstrap peers remain part of Sub-Intent B.

## crates/consensus-simplex
- No code changes in this synthesize pass.
- The only design dependency is that later relay work can now trust `NetworkMessage.channel` to reflect the true mux channel.
- Preserve alignment with `crates/p2p` channel constants; do not introduce crate-local channel IDs.

## crates/app
- No code changes in this synthesize pass.
- Behavioral compatibility is preserved because the provider completeness fixes are below the app boundary.
