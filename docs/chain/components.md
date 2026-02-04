# Components (high level)

Not a spec.

## Node

- P2P (gossip): tx / blocks / votes
- Mempool: pending txs + basic DoS controls
- Execution: tx -> state transition (`state_root`)
- Consensus: propose/vote/finalize
- Storage: blocks + state
- RPC: small query surface

## Roles

- Validator (votes)
- Full node (verifies + serves)
- Optional indexer (derived views)

## Interface shapes

RPC: `get_head`, `get_block`, `submit_tx`, `get_tx`, `subscribe_heads`.

P2P: announce/request for tx, block, vote.
