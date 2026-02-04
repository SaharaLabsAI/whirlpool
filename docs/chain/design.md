# Chain design (high level)

Not a spec. Just the smallest shared mental model.

## Goals

- Deterministic execution + verifiable finalized history.
- Simple separation: P2P, consensus, execution, storage, RPC.

## Core loop

1) Accept txs → 2) build candidate block (execute) → 3) validators vote → 4) finalize head → 5) persist + serve.

## Objects (shape)

- **Block header**: parent, height, time/slot, proposer id, `tx_root`, `state_root`, consensus digest.
- **Block body**: ordered transactions.

## Minimal validity

- Parent is known.
- Execution from parent state is deterministic.
- Roots match computed results.
- Consensus proof meets quorum rules.

## Config surface

- `chain_id`, genesis (validator set + initial state), consensus params, execution limits.

## Trust boundary

Network is adversarial; verify all received objects before storing, voting, or serving.
