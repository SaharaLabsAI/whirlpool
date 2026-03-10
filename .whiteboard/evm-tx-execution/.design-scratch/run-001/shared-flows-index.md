# Shared Flows Index — EVM Transaction Execution

## Primary Flows

### F1: Block Proposal with EVM Execution
- **Trigger**: Consensus engine calls `propose(parent, height)`
- **Actors**: EvmApplication, TxSource, reth BlockBuilder, InMemoryStateDb
- **Happy path**: fetch txs → decode → execute via reth → commit state → assemble EvmBlock
- **Error paths**: tx decode failure (skip tx?), EVM execution error, state commit failure
- **Cross-crate**: app-evm → state (Database reads + commit)

### F2: Block Verification with EVM Re-execution
- **Trigger**: Consensus engine calls `verify(parent, block)`
- **Actors**: EvmApplication, reth BlockExecutor, InMemoryStateDb
- **Happy path**: decode block txs → re-execute → compare results → return Ok
- **Error paths**: tx decode failure, execution mismatch, state root mismatch
- **Cross-crate**: app-evm → state (Database reads, NO commit on verify?)

### F3: State Commitment
- **Trigger**: Successful execution (from propose or verify)
- **Actors**: InMemoryStateDb
- **Happy path**: commit BundleState → update accounts/storage/bytecodes → compute new state_root
- **Error paths**: (currently infallible, but could fail on invalid BundleState)
- **Cross-crate**: state internal only

## Secondary Flows

### F4: Genesis Block Production
- **Trigger**: Node startup, `genesis()` called
- **Actors**: EvmApplication, InMemoryStateDb
- **Happy path**: read current state_root → return empty genesis EvmBlock
- **No changes needed**: genesis is already correct (empty block with initial state_root)

## Flow Dependencies
- F1 depends on F3 (propose must commit state)
- F2 depends on F3 (verify may need state access, open question on commit)
- F4 is independent
