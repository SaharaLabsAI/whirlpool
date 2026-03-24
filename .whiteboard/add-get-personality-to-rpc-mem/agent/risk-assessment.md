# Risk Assessment

## RSK-1: Service boundary expansion in rpc-mem
- Class: resolvable-in-scope
- Description: `MemoryTxService` only supports submit; read endpoint needs either trait expansion or a second trait.
- Resolution: add a read method/trait with explicit result contract and keep submit path untouched.

## RSK-2: Storage source ambiguity (mempool vs finalized)
- Class: resolved-in-scope
- Description: personality reads could incorrectly target pending data.
- Resolution: bind reads to `state::PersonalityStorage::get_latest` semantics.

## RSK-3: Wire schema stability
- Class: accepted
- Description: response may expose byte fields as hex strings and can evolve.
- Acceptance: lock minimal schema for now; defer richer fields/versioning.

## RSK-4: Node wiring complexity
- Class: accepted
- Description: `whirlpool-node` currently constructs a submit-only service.
- Acceptance: scope includes adapter changes but no broader runtime architecture change.

## Expansion
- None proposed.
