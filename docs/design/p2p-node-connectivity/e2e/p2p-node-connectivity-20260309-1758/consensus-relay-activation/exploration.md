# Exploration

## Current State
- `consensus-simplex/src/mailbox.rs`: `Mailbox<B>` is the consensus adapter used by the simplex engine as both the `Automaton` and the `Relay`.
- The `Automaton` side covers proposal, verification, and genesis handling.
- The `Relay` side exposes `broadcast(digest)`, but the current implementation is a no-op. That behavior is sufficient only for single-node execution because no remote validator needs the proposed payload.
- `consensus-simplex/src/engine.rs`: `CommonwareEngine::start()` calls `network.start_per_channel()` and receives the per-channel vote, cert, and resolver pairs.
- `CommonwareEngine::start()` constructs a `Mailbox` and passes it into `simplex::Engine::start(...)` as both automaton and relay, alongside the vendor-managed vote, cert, and resolver channels.
- The vendor simplex engine already handles protocol-level P2P traffic internally through those three channels. Those channels are for consensus protocol messages such as votes, certificates, and resolver requests.
- `p2p-commonware/src/provider.rs` registers the transport channel constants as `VOTE = 0`, `CERT = 1`, and `RESOLVER = 2`.
- `p2p-commonware/src/traits.rs` defines `CommonwareTransport`, whose `start_per_channel()` returns the `PerChannelNetwork` bundle used by the simplex engine.
- `whirlpool-node/src/main.rs` receives an `oracle_handle` from startup wiring, but nothing uses it after startup. As a result, `update_validators()` is never invoked.

## Relay Role in the Consensus Flow
- The simplex relay is not responsible for vote, certificate, or resolver transport. Those paths are already handled by the vendor engine over the existing per-channel network.
- The relay is responsible for application-level content distribution.
- When the engine produces a proposal, it calls `relay.broadcast(digest)` so the relay can distribute the full payload associated with that digest.
- Other validators cannot safely verify or vote on a digest unless they can retrieve the corresponding payload.
- Therefore, relay activation is the missing bridge between proposal creation and remote validator verification in a multi-node deployment.

## Observed Gap
1. `Relay::broadcast()` in the mailbox does nothing, so proposed payloads are never sent to peers.
2. No application-level payload distribution path exists today, even though validators need the full block payload before verification and voting can proceed.
3. The oracle handle is currently unused after startup, so dynamic validator set updates are not part of the active node wiring.

## Required Change Surface
### 1. Replace or wrap the mailbox relay path
- The existing mailbox can continue to serve as the automaton.
- Its relay behavior must be replaced or wrapped so that `broadcast(digest)` performs an actual outbound P2P send.
- This work stays in the application-facing consensus adapter layer rather than vendor simplex internals.

### 2. Use the existing block store as the payload source
- The mailbox already persists proposed payloads via `remember_block()` in its `BlockStore`.
- Because the relay trait only receives a digest, the relay implementation must resolve that digest back to the stored payload before broadcasting.
- The required outbound data source already exists conceptually in the mailbox-side storage path.

### 3. Provide a payload transport path distinct from vendor simplex channels
- The vote, cert, and resolver channels are already consumed by the vendor simplex engine.
- Payload distribution therefore needs its own transport path.
- Based on the current channel allocation, that likely means either a fourth P2P channel or another explicitly separate application-level distribution mechanism.
- Any added path must align with existing transport registration in `p2p-commonware`.

### 4. Persist inbound payloads for local verification
- Receiving nodes must store relayed payloads locally.
- That storage must happen before or by the time local verification asks for the digest contents.
- `Automaton::verify(digest)` depends on being able to find the payload associated with that digest.
- Without inbound storage, remote votes may arrive for digests that the receiving node still cannot verify.

## Alignment-Relevant Conclusions
- The consensus protocol transport is not the missing feature; application payload distribution is.
- The highest-value intervention point is the consensus-simplex relay adapter, because that is where the engine requests payload broadcast.
- The existing mailbox block store provides the required digest-to-payload lookup surface.
- Multi-node consensus requires both outbound payload broadcast and inbound payload persistence.
- Validator update wiring is adjacent context, but the immediate Sub-Intent C gap is relay activation for payload exchange.
