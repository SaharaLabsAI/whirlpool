# Architecture Flows

## Scope
- Sub-Intent A only: `REQ-1`, `REQ-2`, `REQ-3`
- In-scope crates: `crates/p2p-commonware`, `crates/whirlpool-node`
- Stable contract anchor: `crates/p2p`

## Flow 1: Validator Seeding
1. `crates/whirlpool-node/src/main.rs` derives the startup validator set from the node signer/bootstrap state.
2. `crates/whirlpool-node/src/main.rs` calls `CommonwareNetworkProviderBuilder::initial_validators(epoch, validators)` while assembling the P2P provider.
3. `crates/p2p-commonware/src/provider.rs` receives the builder-owned `(epoch, validators)` tuple inside `CommonwareNetworkProviderBuilder::build(context)`.
4. `crates/p2p-commonware/src/provider.rs` constructs `discovery::Network` and clones the discovery oracle into `OracleHandle`.
5. `crates/p2p-commonware/src/provider.rs` calls `oracle_handle.update_validators(epoch, validators.clone()).await` before returning when the validator list is non-empty.
6. `CommonwareNetworkProvider` is handed to consensus startup only after the oracle has the validator baseline needed for admission/discovery.

### Flow Guarantees
- Validator seeding happens exactly once per provider build.
- Empty validator lists do not panic and simply skip the oracle update.
- `crates/whirlpool-node/src/main.rs` supplies the data; `crates/p2p-commonware/src/provider.rs` owns the side effect.

## Flow 2: Bootstrap Discovery
1. `crates/whirlpool-node/src/main.rs` assembles bootstrap peers from node wiring inputs for this pass.
2. `crates/whirlpool-node/src/main.rs` calls `CommonwareNetworkProviderBuilder::bootstrappers(bootstrappers)`.
3. `crates/p2p-commonware/src/provider.rs` threads that list into `discovery::Config::local(signer, namespace, listen_addr, dialable_addr, bootstrappers, max_message_size)`.
4. `discovery::Network::new(context, config)` builds the Commonware discovery runtime using those bootstrappers as discovery seeds.
5. `CommonwareNetworkProvider::start()` or `start_per_channel()` starts the network runtime without reinterpreting bootstrap peers as direct dial targets.
6. Discovery uses the bootstrapper list to surface remote peers beyond the node's direct outbound connections.

### Flow Guarantees
- Bootstrap peers remain separate from static dial peers.
- Empty bootstrapper lists still allow the provider to start.
- No extra discovery layer or vendor modification is introduced.

## Flow 3: Message Routing and Channel Metadata Preservation
1. `crates/p2p-commonware/src/provider.rs` registers mux channels using the stable `crates/p2p` constants:
   - `Channel::VOTE`
   - `Channel::CERTIFICATE`
   - `Channel::RESOLVER`
2. `crates/p2p-commonware/src/provider.rs` creates per-channel receiver adapters by calling:
   - `CommonwareReceiver::new(Channel::VOTE, vote_receiver)`
   - `CommonwareReceiver::new(Channel::CERTIFICATE, cert_receiver)`
   - `CommonwareReceiver::new(Channel::RESOLVER, res_receiver)`
3. A sender transmits bytes over a concrete mux lane in Commonware.
4. The Commonware receiver for that lane yields `(sender_public_key, data)` to `crates/p2p-commonware/src/receiver.rs`.
5. `CommonwareReceiver::recv()` wraps the inbound payload as `NetworkMessage { channel: self.channel, data, peer_id }`.
6. `crates/p2p-commonware/src/lib.rs` `MultiplexReceiver::recv()` returns the already-tagged message to downstream consumers without rewriting the channel.
7. Downstream consumers in later passes can route the message using the preserved `NetworkMessage.channel` value.

### Sender -> Channel -> Receiver Paths
- Vote path: `sender` -> `Channel::VOTE` -> `CommonwareReceiver(Channel::VOTE)` -> `NetworkMessage.channel == Channel::VOTE`
- Certificate path: `sender` -> `Channel::CERTIFICATE` -> `CommonwareReceiver(Channel::CERTIFICATE)` -> `NetworkMessage.channel == Channel::CERTIFICATE`
- Resolver path: `sender` -> `Channel::RESOLVER` -> `CommonwareReceiver(Channel::RESOLVER)` -> `NetworkMessage.channel == Channel::RESOLVER`

### Flow Guarantees
- No receiver may synthesize `Channel(0)` as a placeholder.
- Channel IDs remain defined by `crates/p2p/src/types.rs` and consumed by `crates/p2p-commonware/src/provider.rs`, `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/lib.rs`, and `crates/p2p-commonware/src/sender.rs`.
- This flow unblocks later relay work without changing the `crates/p2p` abstraction.
