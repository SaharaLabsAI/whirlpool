# Tests

## Protected Alignment Baseline

### TST-1 Mixed ingress happy path
- Submit a valid mem/personality transaction through `mem_submitPersonality` and finalize a block.
- Assert the RPC returns a tx hash, the block carries the encoded mem transaction, and the finalized personality store contains the markdown only after finalization.

### TST-2 Oversize markdown rejection
- Submit markdown larger than the consensus-visible limit.
- Assert admission or deterministic validation rejects it and no finalized personality write is produced.

### TST-3 Hash mismatch rejection
- Submit a payload whose declared markdown hash does not match markdown bytes.
- Assert proposal/verification rejects the mem transaction deterministically.

### TST-4 Mixed block preservation
- Propose and verify a block with both one valid EVM transaction and one valid mem transaction.
- Assert EVM execution semantics stay unchanged and mem validation remains structural/deterministic.

### TST-5 Finalization-only visibility
- Inspect the personality store before and after finalization.
- Assert the store is unchanged before finalization and updated only by finalization handling.

### TST-6 Replacement semantics
- Finalize two mem transactions for the same `personality_id` in later blocks.
- Assert the store returns the later markdown as the active value.

### TST-7 Prototype volatility documentation
- Restart the node after a finalized mem write.
- Assert the in-memory store is empty after restart and the behavior is documented.
