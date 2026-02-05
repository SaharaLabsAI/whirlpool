# `consensus` — Simplex wiring (commonware)

This chain uses `vendor/commonware/consensus` (Simplex) as the consensus engine.

Goal: keep chain logic behind `core` traits, and make `consensus` a thin adapter that
wires networking + storage + runtime into the commonware engine.

## Trait mapping (our `core` ↔ commonware)

Your chain-specific state transition rules live in an application object that implements:

- `core::ConsensusApplication` (genesis + propose)
- `core::VerifyingApplication` (verify)
- optionally `core::Reporter<Activity>` (receive finalized notifications)

The adapter layer implements the corresponding commonware traits:

- `commonware_consensus::Application<E>` ↔ `core::ConsensusApplication`
- `commonware_consensus::VerifyingApplication<E>` ↔ `core::VerifyingApplication`
- `commonware_consensus::Reporter` ↔ `core::Reporter`

In `vendor/alto/chain`, this is done by implementing commonware traits directly on the
chain `Application` and (optionally) `Indexer/Pusher`.

## Components you need to wire (Alto pattern)

Simplex is not a single struct you call; it is composed with a few surrounding pieces.
The canonical wiring example in this repo is `vendor/alto/chain/src/engine.rs`.

High-level components:

1. **Buffered broadcast / relay plane**
   - A gossip/broadcast helper used by consensus for proposals/votes.

2. **Marshaling layer**
   - A marshal actor + mailbox used to fetch/subscribe to proposal payloads.
   - Commonware provides a wrapper (`commonware_consensus::application::marshaled::Marshaled`)
     that couples the app with a marshal mailbox/resolver.
   - In Alto, some structural/linking invariants are enforced by this layer.

3. **`commonware_consensus::simplex::Engine`**
   - Constructed with `simplex::Config { scheme, elector, blocker, automaton, relay, reporter, ... }`.
   - Started alongside the buffered broadcast and marshal tasks.

4. **Reporter (events out)**
   - A `commonware_consensus::Reporter` implementation that converts engine activity into chain events.
   - Minimal output we care about: `ConsensusEvent::Finalized(types::FinalizedBlock)`.

## Control flow (end-to-end)

1. **Startup**
   - Build P2P channels used by commonware (pending votes / recovered certs / resolver+fetch).
   - Build buffered broadcast.
   - Start marshal actor and wrap the app as a `Marshaled` application.
   - Build Simplex engine with config + relay + reporter.
   - Run broadcast + marshal + simplex concurrently.

2. **Propose**
   - Engine calls `Application::propose(...)`.
   - Your app builds a candidate `types::Block` (height = parent.height + 1, timestamp monotonic, txs signed).

3. **Verify**
   - Engine calls `VerifyingApplication::verify(...)` against an ancestry stream.
   - Your app checks semantic validity; marshaling may enforce structural invariants.

4. **Finalize**
   - When quorum is reached, Simplex finalizes a block and calls `Reporter::report(...)`.
   - The consensus crate converts that into `ConsensusEvent::Finalized(types::FinalizedBlock)`.

## Where certificates/proofs attach

Blocks are sealed. Consensus produces certificates/proofs alongside blocks.
Emit a `types::FinalizedBlock { certificate, block }` (or family-specific equivalent).
This mirrors the Alto pattern: the reporter receives finalization activity and can optionally
fetch the referenced block bytes via marshal subscription before emitting/uploading artifacts.
