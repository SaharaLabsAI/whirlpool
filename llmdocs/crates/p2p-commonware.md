# p2p-commonware Crate

## Purpose
Bridge the vendor-agnostic `p2p` interfaces to the Commonware networking stack.

## Interface/Implementation Split
- Interface module: `crates/p2p-commonware/src/traits.rs`
  - `CommonwareTransport`
- Implementation modules:
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/p2p-commonware/src/sender.rs`
  - `crates/p2p-commonware/src/receiver.rs`
  - `crates/p2p-commonware/src/lib.rs` (`MultiplexSender`, `MultiplexReceiver`)

## Trait Boundary
`CommonwareTransport` provides an interface boundary for providers that expose dedicated simplex channels:
- `start_per_channel(self) -> Result<PerChannelNetwork<...>, P2pError>`
- `oracle(&self) -> &Oracle<_>`

`CommonwareNetworkProvider` implements both:
- `p2p::traits::NetworkProvider` (multiplexed sender/receiver)
- `CommonwareTransport` (dedicated vote/cert/resolver/payload channels)

## Channel Layout
`PerChannelNetwork` holds 4 dedicated channel pairs (sender, receiver):
- `vote`: `Channel::VOTE` (0) — consensus vote messages
- `cert`: `Channel::CERT` (1) — certificate messages
- `resolver`: `Channel::RESOLVER` (2) — block resolver messages
- `payload`: `Channel::PAYLOAD` (3) — block payload relay messages

Channel constants defined in `crates/p2p/src/types.rs` as `Channel(N)` associated constants.

## Canonical Imports
- `p2p_commonware::traits::CommonwareTransport`
- `p2p::traits::{NetworkProvider, NetworkSender, NetworkReceiver, PeerId}`

## Key Types
- `CommonwareNetworkProviderBuilder`
- `CommonwareNetworkProvider`
- `OracleHandle`
- `MultiplexSender`, `MultiplexReceiver`
- `CommonwarePeerId`
- `PerChannelNetwork` (with `vote`, `cert`, `resolver`, `payload` fields)

## Status
Complete. Transport interface is explicitly separated into `traits.rs`. PAYLOAD channel (3) registered for consensus relay.
