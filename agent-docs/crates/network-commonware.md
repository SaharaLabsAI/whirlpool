# network-commonware Crate

## Purpose
Bridge the vendor-agnostic `network` interfaces to the Commonware networking stack.

## Interface/Implementation Split
- Interface module: `crates/network/commonware/src/traits.rs`
  - `CommonwareTransport`
- Implementation modules:
  - `crates/network/commonware/src/provider.rs`
  - `crates/network/commonware/src/sender.rs`
  - `crates/network/commonware/src/receiver.rs`
  - `crates/network/commonware/src/lib.rs` (`MultiplexSender`, `MultiplexReceiver`)

## Trait Boundary
`CommonwareTransport` provides an interface boundary for providers that expose dedicated simplex channels:
- `start_per_channel(self) -> Result<PerChannelNetwork<...>, P2pError>`
- `oracle(&self) -> &Oracle<_>`

`CommonwareNetworkProvider` implements both:
- `network::traits::NetworkProvider` (multiplexed sender/receiver)
- `CommonwareTransport` (dedicated vote/cert/resolver/payload channels)

## Channel Layout
`PerChannelNetwork` holds 4 dedicated channel pairs (sender, receiver):
- `vote`: `Channel::VOTE` (0) — consensus vote messages
- `cert`: `Channel::CERT` (1) — certificate messages
- `resolver`: `Channel::RESOLVER` (2) — block resolver messages
- `payload`: `Channel::PAYLOAD` (3) — block payload relay messages

Channel constants defined in `crates/network/traits/src/types.rs` as `Channel(N)` associated constants.

## Canonical Imports
- `network_commonware::traits::CommonwareTransport`
- `network::traits::{NetworkProvider, NetworkSender, NetworkReceiver, PeerId}`

## Key Types
- `CommonwareNetworkProviderBuilder`
- `CommonwareNetworkProvider`
- `OracleHandle`
- `MultiplexSender`, `MultiplexReceiver`
- `CommonwarePeerId`
- `PerChannelNetwork` (with `vote`, `cert`, `resolver`, `payload` fields)
- `MultiplexReceiver::recv` scans each registered receiver once per call and returns the first available tagged message

## Commonware 2026.4.0 Compatibility Notes
- Discovery network contexts now require `commonware_runtime::BufferPooler` in addition to spawner/clock/network traits.
- `OracleHandle::update_validators` now uses `commonware_p2p::Manager::track` with a deduplicated `commonware_utils::ordered::Set` (the old `Manager::Peers`/`update` path is removed upstream).
- Commonware receivers now yield `IoBuf`; `CommonwareReceiver` converts to `bytes::Bytes` before emitting `NetworkMessage`.

## Status
Complete. Transport interface is explicitly separated into `traits.rs`. PAYLOAD channel (3) registered for consensus relay.
Clippy hygiene: provider channel registration reuses `Quota` by value (copy semantics); unit-test module is named `network_commonware_tests` to avoid module-inception lint.
