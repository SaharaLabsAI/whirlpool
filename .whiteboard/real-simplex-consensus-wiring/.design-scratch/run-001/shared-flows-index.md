# Shared Flows Index

## Flow 1: Engine Startup

**Actors**: whirlpool-node main → CommonwareEngine → P2P provider → Mailbox/Actor → simplex::Engine
**Trigger**: `engine.start()` called from `main.rs`
**Steps**: Network start → Component creation → Simplex config → Engine start → RunningEngine returned
**Error paths**: Network failure, config validation failure, engine start failure

## Flow 2: Block Production (Propose)

**Actors**: simplex::Engine → Mailbox (Automaton) → MailboxActor → ConsensusApp → EvmApplication
**Trigger**: simplex selects current node as leader for a view
**Steps**: Engine calls automaton.propose() → Mailbox sends to Actor → Actor calls app.propose() → EvmApplication executes txs via reth → returns EvmBlock
**Error paths**: No pending txs (empty block), EVM execution failure, timeout

## Flow 3: Block Verification (Verify)

**Actors**: simplex::Engine → Mailbox (Automaton) → MailboxActor → ConsensusApp → EvmApplication
**Trigger**: simplex receives proposed block from leader
**Steps**: Engine calls automaton.verify() → Mailbox sends to Actor → Actor calls app.verify() → EvmApplication re-executes and compares roots
**Error paths**: State root mismatch, tx root mismatch, receipts root mismatch

## Flow 4: Block Finalization (Report)

**Actors**: simplex::Engine → AppAdapter (Reporter) → EventSink → FinalizationSink
**Trigger**: simplex achieves consensus (2f+1 votes)
**Steps**: Engine calls reporter.report(Update::Block(block, ack)) → AppAdapter emits ConsensusEvent::Finalized → EventSink handles → FinalizationSink stores height → ack.acknowledge()
**Error paths**: Sink failure (logged), ack failure

## Flow 5: Engine Shutdown

**Actors**: RunningEngine → Handle<()> abort → simplex actors stop → P2P stop
**Trigger**: `running.shutdown()` called
**Steps**: Abort simplex Handle → actors drain → network closes
**Error paths**: Actor panic, network cleanup timeout
