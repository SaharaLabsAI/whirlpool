# Design Review

## Overview
Add a new non-EVM transaction family for agent personality submission. The transaction carries markdown content that defines an agent personality profile. In v1, the chain only needs to accept the transaction, carry it through the existing node pipeline, and persist the personality content into a dedicated in-memory store backed by a `HashMap`. Execution side effects, agent activation semantics, and Jolt-based signature verification are explicitly deferred.

## Problem Statement
Whirlpool currently accepts EVM transactions through the reth-backed `eth_sendRawTransaction` path and executes them inside `EvmApplication`. There is no first-class transaction type for application-defined content such as agent personality markdown. We need a formal design for introducing a new transaction type that:
- enters through the JSON-RPC layer,
- can sit in the mempool,
- is included in proposed blocks,
- is validated deterministically during block verification, and
- persists the submitted personality markdown on finalization.

The prototype may require new storage for this transaction. For the prototype, an in-memory `HashMap` is sufficient.

## Goals
- Define a new signed transaction family for personality markdown submission.
- Preserve the current layered architecture: RPC -> mempool -> consensus/app -> finalization storage.
- Keep consensus behavior deterministic across proposal and verification.
- Persist personality content only after block finalization.
- Introduce a dedicated logical personality store with a prototype in-memory `HashMap` backend.
- Leave a clean boundary for future Jolt zkVM-based signature verification.

## Non-Goals
- No personality execution semantics inside the EVM.
- No durable personality storage in v1.
- No retrieval/query RPC beyond what is needed to submit transactions.
- No markdown rendering, sanitization policy, or agent runtime behavior.
- No mandatory Jolt proof generation or proof verification in v1.

## Current Architecture Summary
- `crates/whirlpool-node/src/node.rs`: wires RPC, mempool, EVM app, consensus engine, and finalization sink.
- `crates/rpc-eth/src/pool.rs`: `WhirlpoolTxPool` accepts Ethereum transactions from reth RPC, rejects blob txs, encodes them as EIP-2718 bytes, and forwards them to `TxSource::push`.
- `crates/app/src/traits.rs`: `TxSource` is the current mempool boundary: `push(tx: Vec<u8>)` and `pending() -> Vec<Vec<u8>>`.
- `crates/mempool-mdbx/src/persistent.rs`: production `TxSource` backed by MDBX.
- `crates/app-evm/src/executor.rs`: `EvmApplication::propose` drains `TxSource`, decodes EVM transactions, executes valid ones, and builds an `EvmBlock`; `verify` re-executes block transactions and checks roots.
- `crates/whirlpool-node/src/persisting_sink.rs`: finalized blocks are persisted only after consensus finalization.
- `crates/state/src/block_storage.rs`: `BlockStorage` persists finalized blocks and receipts, but does not expose a store for application-specific metadata such as personality content.
- `docs/chain/crates/types.md`: high-level design already allows multiple transaction families via `SignedTransaction::Other { type_id, payload }`.

Today, the node runs a single Ethereum-oriented JSON-RPC surface. The new personality transaction is still a natural fit as the first real non-EVM transaction family, but it should enter through its own experimental RPC stack rather than the Ethereum one.

## Proposed Transaction Family

### Transaction Kind
Introduce a new non-EVM signed transaction family:

```text
SignedTransaction::Other {
  type_id: PERSONALITY_MARKDOWN_TX,
  payload: <family-defined bytes>
}
```

Define a concrete payload shape conceptually equivalent to:

```rust
struct PersonalityMarkdownTx {
    version: u8,
    signer: Vec<u8>,
    personality_id: Vec<u8>,
    nonce: u64,
    markdown_bytes: Vec<u8>,
    markdown_hash: [u8; 32],
    signature_scheme: SignatureScheme,
    signature: Vec<u8>,
}
```

### Rationale
- `docs/chain/crates/types.md` already reserves `Other { type_id, payload }` as the extension point for non-EVM transaction families.
- A dedicated family avoids overloading EVM transaction encoding with non-EVM semantics.
- The payload can be validated deterministically without requiring EVM execution.
- The `signer` and `signature` fields keep the format forward-compatible with Jolt-based verification.

## Canonical Semantics
A personality-markdown transaction means:
- the signer submits a complete markdown document,
- the chain binds that document to the signer identity,
- once the containing block is finalized, the document is written to the personality store,
- the latest finalized personality for a signer or `personality_id` replaces the previous one in the prototype store.

For the prototype, the state model is:
- one active personality document per `personality_id`,
- last-finalized write wins,
- optional lookup by `(signer, nonce)` for replay protection,
- no deletion transaction.

## Data Model

### Required Fields
- `version`: payload version for future upgrades.
- `signer`: identity key used to namespace writes.
- `personality_id`: stable identity for the personality entry.
- `nonce`: replay/version slot for future use.
- `markdown_bytes`: raw UTF-8 markdown document.
- `markdown_hash`: content digest over canonical bytes.
- `signature_scheme`: reserved for future verification policy.
- `signature`: signature material or proof-bearing wrapper.

### Canonicalization Rules
For v1, keep canonicalization minimal and deterministic:
- `markdown_bytes` must be valid UTF-8.
- The exact submitted bytes are the persisted bytes.
- `markdown_hash` must equal the hash of the exact markdown bytes.
- No whitespace normalization, markdown parsing, or semantic validation.

### Size Limits
Define an explicit consensus-visible byte limit, for example:
- `MAX_PERSONALITY_MARKDOWN_BYTES = 16 * 1024`

The exact value can be tuned later, but the design requires a hard limit so all nodes reject oversize payloads identically.

## RPC Design

### Recommendation
Add a separate experimental RPC stack in a new crate:

```text
crates/rpc-mem
```

`rpc-mem` owns non-EVM personality or memory-facing methods and runs as a second JSON-RPC server started by `whirlpool-node`. `rpc-eth` remains Ethereum-only.

Recommended external method in `rpc-mem`:

```text
mem_submitPersonality
```

This keeps unstable experimentation contained in one crate while preserving a clean compatibility boundary for Ethereum RPC.

### Why a Separate `rpc-mem` Crate
- `rpc-eth` is currently reth-backed and built around Ethereum transaction pool traits.
- `eth_sendRawTransaction` expects Ethereum transaction envelopes and routes through `WhirlpoolTxPool` in `crates/rpc-eth/src/pool.rs`.
- A personality transaction is not an Ethereum typed transaction and should not pretend to be one.
- The personality or memory feature set is expected to evolve quickly and may become messy during experimentation.
- A dedicated crate keeps experimental RPC methods, request types, validation, and wiring contained.
- `whirlpool-node` can expose both servers side by side without mixing their implementation concerns.

### Server Topology
`whirlpool-node` should start two RPC servers in the same node process:
- Ethereum server from `crates/rpc-eth` for EVM-compatible methods.
- Memory server from `crates/rpc-mem` for personality submission and future memory-specific queries.

This is a logical separation, not a separate node role. Both servers talk to the same underlying chain and mempool environment, but through different service adapters.

### Request Shape

```json
{
  "version": 1,
  "signer": "0x...",
  "personality_id": "0x...",
  "nonce": 1,
  "markdown": "# Persona\nYou are a careful agent...",
  "signatureScheme": "raw_secp256k1",
  "signature": "0x..."
}
```

### Response Shape

```json
{
  "txHash": "0x..."
}
```

### RPC Admission Rules
The `rpc-mem` method should:
- validate request shape,
- encode the transaction into canonical non-EVM payload bytes,
- compute a transaction hash,
- enqueue the payload into the mempool ingress path,
- return the hash if accepted.

Possible rejection reasons:
- invalid UTF-8 markdown,
- empty markdown,
- markdown exceeds configured byte limit,
- unsupported payload version,
- unsupported signature scheme,
- malformed signature bytes,
- hash mismatch,
- duplicate pending tx if deduplication is added.

### Recommended Internal Boundary
Keep `rpc-mem` isolated from direct mempool details through a service trait such as:

```rust
trait MemoryTxService {
    fn submit_personality(&self, req: SubmitPersonalityRequest) -> Result<[u8; 32], MemoryTxError>;
}
```

An adapter owned by node wiring can implement this trait on top of the generic transaction ingress path. This keeps `rpc-mem` self-contained even if the underlying mempool stays shared with EVM transactions.


## Mempool Design

### Current Baseline
`TxSource` stores opaque `Vec<u8>` blobs. That is useful because the personality transaction can initially flow through the same abstraction.

### Proposed Change
Broaden the transaction source abstraction from "raw EVM bytes" to "raw signed transaction bytes". The mempool remains payload-agnostic.

In v1, the mempool should:
- accept encoded personality transaction bytes,
- preserve FIFO behavior,
- return raw bytes during `pending()`,
- avoid semantic mutation.

### Recommended Direction
Keep the mempool generic and do not create a separate personality-only queue yet. The application layer should classify pending items into:
- EVM transactions,
- personality-markdown transactions,
- invalid or unknown transactions.

### Deduplication and Replay
For the prototype, minimal policy is acceptable:
- optionally dedup by tx hash while pending,
- do not enforce complex signer replacement rules in mempool,
- reject finalized duplicate `(signer, nonce)` if replay protection is enabled in the storage-backed validation path.

## Application and Consensus Design

### Key Observation
`EvmApplication` currently assumes every transaction in `TxSource` is EVM-decodable and executable. That assumption must be relaxed.

### Proposed Architectural Split
Introduce a block transaction model with two transaction classes inside the block body:
- EVM transactions: executed through the existing EVM path.
- Personality transactions: validated structurally, included in the block, but not executed in the EVM.

Conceptually:

```rust
enum WhirlpoolSignedTx {
    Evm(Vec<u8>),
    Personality(PersonalityMarkdownTx),
}
```

The block may still physically store encoded bytes, but `propose` and `verify` must classify each item consistently.

### Proposal Rules
During `propose(parent, height)`:
1. Drain mempool bytes via `TxSource::pending()`.
2. Decode each raw item into either EVM tx, personality tx, or invalid.
3. For EVM txs, use existing execution logic and include successfully executed tx bytes in the block.
4. For personality txs, perform structural validation only, include accepted tx bytes in the block, and accumulate derived writes in a pending side-channel.
5. Build block roots over the ordered list of accepted encoded transactions.
6. Do not write personality content to storage during proposal.

### Verification Rules
During `verify(parent, block)`:
1. Decode every block transaction.
2. Re-execute EVM transactions exactly as today.
3. Re-validate every personality transaction using deterministic structural rules.
4. Reconstruct the derived set of pending personality writes from the block contents.
5. Verify block transaction root and any other commitments still match.
6. Store the derived personality writes in a temporary cache, analogous to `pending_receipts`.

### Deterministic Validation Rules for Personality Tx
All validators must apply the same checks:
- payload version is supported,
- payload decodes successfully,
- signer field is present and properly encoded,
- personality ID is present,
- markdown is valid UTF-8,
- markdown is non-empty,
- markdown length is within the configured maximum,
- `markdown_hash` matches the markdown bytes,
- signature field is present and structurally well-formed for the declared scheme.

In v1, signature correctness is not proven through Jolt yet. Verification policy must be explicit.

### Recommended v1 Verification Policy
Use a two-tier rule set:
- consensus-critical now: payload format, size, UTF-8 validity, hash integrity, and signature field presence or encoding,
- deferred: cryptographic proof that the signer authorized the payload.

This keeps all nodes deterministic today without blocking the prototype on zkVM integration.

### Consensus Caveat
The architecture exploration found that consensus mailbox verification in `crates/consensus-simplex/src/mailbox.rs` is mostly digest and payload integrity, not full app re-execution. The proposal should therefore preserve the current behavior but note that stronger app-level verification integration may be required if the project later wants consensus-layer enforcement stronger than payload digest validity.

## Finalization and Storage Design

### Persistence Point
Persist personality content only on finalization, matching the current block persistence model in `crates/whirlpool-node/src/persisting_sink.rs`.

This is the most important safety property in the design:
- mempool admission is not canonical,
- proposal is speculative,
- verification is preparatory,
- finalization is the first point where personality data becomes canonical node state.

### New Logical Store
Add a dedicated logical store for personality state, separate from `BlockStorage`.

Recommended trait:

```rust
trait PersonalityStorage: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn put(&self, entry: StoredPersonality) -> Result<(), Self::Error>;
    fn get_latest(&self, personality_id: &[u8]) -> Result<Option<StoredPersonality>, Self::Error>;
    fn get_by_signer_nonce(
        &self,
        signer: &[u8],
        nonce: u64,
    ) -> Result<Option<StoredPersonality>, Self::Error>;
}
```

Optional stored value:

```rust
struct StoredPersonality {
    tx_hash: [u8; 32],
    block_height: u64,
    signer: Vec<u8>,
    personality_id: Vec<u8>,
    nonce: u64,
    markdown: Vec<u8>,
    markdown_hash: [u8; 32],
}
```

### Prototype Backend
Use an in-memory `HashMap` protected by a lock.

Recommended prototype implementation:

```rust
struct InMemoryPersonalityStorage {
    by_personality_id: RwLock<HashMap<Vec<u8>, StoredPersonality>>,
    by_signer_nonce: RwLock<HashMap<(Vec<u8>, u64), Vec<u8>>>,
}
```

This is enough to:
- persist finalized personality content in memory,
- reject replay of already-finalized `(signer, nonce)` if that policy is enabled,
- read the latest personality by `personality_id`.

### Ownership and Wiring
The personality store should be owned by the node wiring layer alongside the existing block storage.

Recommended placement:
- instantiate the store in `crates/whirlpool-node/src/node.rs`,
- instantiate a second RPC server from `crates/rpc-mem` in `crates/whirlpool-node/src/node.rs`,
- pass the store into the application or finalization pipeline,
- pass a memory-specific submission adapter into `rpc-mem`,
- extend `PersistingFinalizationSink` or a sibling sink so that finalized personality writes are flushed together with block finalization handling.

Recommended node-level ownership model:
- `rpc-eth` continues to own Ethereum-facing RPC wiring only,
- `rpc-mem` owns personality or memory RPC wiring only,
- `whirlpool-node` owns both server lifecycles and their shared dependencies,
- the mempool may remain shared underneath, but `rpc-mem` should reach it through a contained memory-ingress adapter rather than raw direct access.

### Write Path
On finalization of a block:
1. read the derived pending personality writes associated with that block,
2. write each entry into `PersonalityStorage`,
3. then delegate to the inner finalization sink.

### Failure Policy
The current `PersistingFinalizationSink` logs storage failures and continues finalization. For personality storage, make the policy explicit.

Recommendation for prototype:
- log failures and continue, matching current block-persistence error handling.

Recommendation for future productionization:
- revisit this policy because silent divergence in personality storage may be unacceptable once the data becomes user-visible state.

## Storage Semantics
For v1, define the store semantics precisely:
- key: `personality_id`, with optional secondary key `(signer, nonce)`,
- value: latest finalized personality markdown for that entry,
- overwrite policy: last-finalized write wins,
- visibility: data is visible only after finalization,
- restart behavior: prototype data is lost on node restart,
- historical retention: not required.

This must be called out clearly so users understand that v1 is a prototype state surface, not durable chain state.

## Jolt zkVM Integration Plan
The Jolt documentation review shows that Jolt provides proving and verification building blocks, guest-host workflows, advice patterns, and crypto inlines, but it does not define a native transaction-family model for personality markdown. Whirlpool should therefore define the application-layer transaction schema first.

Relevant Jolt guidance from the reviewed docs:
- Jolt is suited to proving computation, not defining mempool protocol structure.
- Signature-related patterns point to signature recovery examples and crypto inline support.
- Jolt docs mention practical input-size constraints and explicit proof-verifier boundaries.
- Large or host-supplied data can use advice-style integrity checks.

### Future Verification Boundary
In a later phase, replace v1 signature-structure checks with proof-backed verification that:
- the signer authorized the payload,
- the verified message binds at least `version`, `personality_id`, `nonce`, `signer`, and `markdown_hash`,
- the proof is valid under the declared verification circuit.

### Recommended Future Message to Verify
Use a domain-separated digest such as:

```text
H("whirlpool/personality/v1" || chain_id || version || signer || personality_id || nonce || markdown_hash)
```

This avoids signing raw markdown directly while still binding the exact content hash.

### Why Jolt Is Deferred in v1
- The current objective is storage-only behavior.
- Jolt integration requires explicit circuit and interface design, host-guest plumbing, and proof-size or runtime considerations.
- None of that is necessary to formalize the transaction lifecycle and storage semantics now.

## Security and Abuse Considerations
- Spam risk: markdown payloads are larger than typical metadata; enforce byte limits at RPC and verification.
- Malformed payloads: must fail deterministically during decode.
- Signature ambiguity: v1 must not imply cryptographic authenticity beyond structural checks.
- Unicode edge cases: v1 stores exact UTF-8 bytes and does not normalize.
- Replay or versioning: if nonce enforcement is not enabled immediately, document that clearly as a limitation.
- Memory growth: in-memory `HashMap` can grow unbounded if every signer is unique; prototype deployments should use conservative limits.

## Observability
Add logs and metrics at each layer.

Recommended signals:
- RPC accepted and rejected personality submission counts,
- mempool queued personality tx count and payload byte totals,
- proposal included personality tx count per block,
- verification rejected personality tx count by reason,
- finalization persisted personality write count,
- storage current number of stored personalities.

## Rollout Plan

### Phase 1: Prototype
- Add transaction schema and encoding.
- Add a dedicated `crates/rpc-mem` server and `mem_submitPersonality` method.
- Extend `whirlpool-node` to start both `rpc-eth` and `rpc-mem`.
- Extend mempool and application flow to carry the new transaction family.
- Add in-memory `HashMap`-backed `PersonalityStorage`.
- Persist on finalization only.
- Do structural validation only.

### Phase 2: Hardening
- Add replay policy and signer or version semantics.
- Add durable storage backend.
- Add retrieval or query RPC.
- Improve deduplication and mempool replacement rules.

### Phase 3: Jolt Verification
- Define Jolt guest-host proof boundary.
- Bind proof to canonical message digest.
- Replace structural signature checks with proof verification.
- Add proof-specific limits and observability.

## Test Plan
### Objective
Verify that the new personality transaction is accepted, propagated through the existing pipeline, and persisted to the prototype personality store only after finalization.

### Prerequisites
- Node wiring can construct a personality store instance.
- Application path can classify EVM and personality transactions.
- `rpc-mem` submission RPC exists and is started by `whirlpool-node`.

### Test Cases
1. Happy path submission: submit valid markdown tx -> tx accepted -> block includes tx -> finalization writes entry -> verify storage returns submitted markdown.
2. Oversize markdown rejection: submit payload above max size -> RPC or verify rejects -> no finalized write.
3. Malformed payload rejection: invalid encoded transaction -> rejected deterministically -> no finalized write.
4. Hash mismatch rejection: markdown bytes do not match declared `markdown_hash` -> verify fails -> block invalid.
5. Replacement semantics: same `personality_id` submits two finalized personality txs in later blocks -> store returns the later markdown.
6. Prototype volatility: restart node -> in-memory personality store is empty again -> behavior documented and expected.

### Success Criteria
- All nodes derive the same accepted and rejected personality writes from the same block.
- Personality data is not persisted before finalization.
- Finalized writes update the prototype store deterministically.
- EVM transaction behavior remains unchanged.

### How To Execute
- Unit tests for payload codec and validation.
- Unit tests for in-memory `HashMap` personality store.
- Unit tests for `rpc-mem` request validation and service adapter boundaries.
- Application tests for mixed EVM and personality block proposal and verification.
- Integration test for `rpc-mem` submission through finalization.

## Open Questions
- Should signer identity use Ethereum address format, ed25519 public key bytes, or a chain-specific identity wrapper?
- Should the prototype enforce one pending tx per signer in mempool, or allow multiple and rely on finalization order?
- Should the transaction hash be derived from the raw encoded payload or from a canonical field digest?
- Should stored personality values retain block height and tx hash in the prototype store, or only markdown content?
- Should `rpc-mem` also own read methods such as `mem_getPersonality` in the first milestone, or remain submit-only?
- Should `rpc-mem` bind to a separate address and port from `rpc-eth`, or share a listener through a multiplexer in a later phase?

## Recommended Decisions
To unblock implementation, the following decisions are recommended now:
- Add a separate `crates/rpc-mem` crate for experimental personality or memory RPC methods.
- Start a second RPC server from `crates/whirlpool-node/src/node.rs` and keep `rpc-eth` Ethereum-only.
- Use `mem_submitPersonality` as the initial external method.
- Treat personality markdown as a non-EVM transaction family using the existing `Other { type_id, payload }` extension concept.
- Keep the mempool generic and payload-agnostic underneath.
- Add a memory-ingress adapter between `rpc-mem` and the shared transaction source.
- Persist personality content only on finalization.
- Add a dedicated `PersonalityStorage` abstraction with an in-memory `HashMap` backend for the prototype.
- Use last-finalized-write-wins semantics per `personality_id`.
- Mark Jolt verification as a future phase with a clear message-binding plan.

## Completeness Check
- [x] JSON-RPC ingress covered.
- [x] Mempool behavior covered.
- [x] Proposal path covered.
- [x] Verify path covered.
- [x] Finalization storage semantics covered.
- [x] Prototype `HashMap` storage covered.
- [x] Jolt future integration covered.
- [x] Explicit goals and non-goals covered.

## Verdict
The proposal fits Whirlpool's existing architecture. The smallest clean implementation is to introduce a new non-EVM signed transaction family, expose it through a dedicated `crates/rpc-mem` server with `mem_submitPersonality`, keep the mempool as an opaque byte queue behind a memory-ingress adapter, classify the new payload in application proposal and verification, and persist finalized personality content into a dedicated in-memory `HashMap` store behind a new storage trait.
