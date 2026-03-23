# Tests

## Protected Alignment Baseline

### TST-1 Mixed ingress happy path
- Environment: single node with shared mempool, `rpc-eth`, and new `rpc-mem` enabled.
- Action: submit a valid mem/personality transaction through `mem_submitPersonality`, allow proposal and finalization.
- Assertions: RPC returns tx hash, block includes the encoded mem transaction, finalized personality store contains the submitted markdown only after finalization.

### TST-2 Oversize markdown rejection
- Environment: same as TST-1.
- Action: submit markdown larger than the consensus-visible limit.
- Assertions: RPC or deterministic validation rejects the payload, no finalized personality write is produced.

### TST-3 Hash mismatch rejection
- Environment: same as TST-1.
- Action: submit a payload whose declared markdown hash does not match markdown bytes.
- Assertions: proposal/verification rejects the mem transaction deterministically and block acceptance fails or transaction is excluded by rule.

### TST-4 Mixed block preservation
- Environment: node with at least one valid EVM transaction and one valid mem transaction pending.
- Action: propose and verify a block with both transaction families.
- Assertions: EVM execution behavior matches current semantics, mem transaction is structurally validated, and ordering/roots stay deterministic.

### TST-5 Finalization-only visibility
- Environment: node with mem transaction accepted into mempool but block not yet finalized.
- Action: inspect prototype personality store before and after finalization.
- Assertions: store is unchanged before finalization and updated only after finalization sink handling.

### TST-6 Replacement semantics
- Environment: same node, same `personality_id`, two finalized mem transactions in later blocks.
- Action: finalize both blocks in order.
- Assertions: store returns the later markdown as the active value for the `personality_id`.

### TST-7 Prototype volatility documentation
- Environment: node restart after finalized mem write.
- Action: restart and inspect the prototype store.
- Assertions: in-memory store is empty after restart and documentation/test expectations call this out explicitly.
