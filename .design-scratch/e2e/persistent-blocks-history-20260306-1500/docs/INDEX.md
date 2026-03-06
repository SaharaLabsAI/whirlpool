# Design: Persistent Block Storage & History Queries

Implement durable finalized block persistence and history queries so Whirlpool can serve `eth_getBlockByHash`/`eth_getBlockByNumber` from MDBX-backed data across restarts.

## File inventory

| File | Lines |
|------|------:|
| BLOCKERS.md | 85 |
| CRATES.md | 175 |
| DOMAINS.md | 692 |
| EXPLORATION.md | 95 |
| EXPLORATION_DIGEST.md | 21 |
| FLOWS.md | 291 |
| INTENT.md | 41 |
| RISK_TRIAGE.md | 42 |
| SHARED_CONTEXT.md | 33 |
| STRATEGY.md | 282 |
| TESTS.md | 94 |
| WORKSPACE.md | 284 |
| crates/app-evm.md | 227 |
| crates/app.md | 57 |
| crates/consensus-simplex.md | 57 |
| crates/rpc-eth.md | 321 |
| crates/state-reth.md | 241 |
| crates/state.md | 107 |
| crates/whirlpool-node.md | 146 |

## Loading tiers

### Tier 1 — Read first (essential for understanding)

| File | Purpose | Lines |
|------|---------|------:|
| INTENT.md | Problem statement, scope, and success criteria | 41 |
| STRATEGY.md | End-to-end implementation approach and sequencing | 282 |
| CRATES.md | Crate-level ownership and boundaries | 175 |
| FLOWS.md | Cross-crate execution flows and integration slices | 291 |

### Tier 2 — Read for detail

| File | Purpose | Lines |
|------|---------|------:|
| DOMAINS.md | Domain responsibilities and contracts in depth | 692 |
| WORKSPACE.md | Workspace structure and dependency-level context | 284 |
| TESTS.md | Test contracts, coverage strategy, and validation plan | 94 |
| BLOCKERS.md | Open risks, blockers, and resolution tracking | 85 |

### Tier 3 — Reference (on-demand)

| File | Purpose | Lines |
|------|---------|------:|
| EXPLORATION.md | Raw exploration notes and source observations | 95 |
| EXPLORATION_DIGEST.md | Condensed exploration highlights | 21 |
| SHARED_CONTEXT.md | Shared assumptions and aligned terminology | 33 |
| RISK_TRIAGE.md | Risk ranking and mitigation framing | 42 |
| crates/app-evm.md | Per-crate contract/reference details | 227 |
| crates/app.md | Per-crate contract/reference details | 57 |
| crates/consensus-simplex.md | Per-crate contract/reference details | 57 |
| crates/rpc-eth.md | Per-crate contract/reference details | 321 |
| crates/state-reth.md | Per-crate contract/reference details | 241 |
| crates/state.md | Per-crate contract/reference details | 107 |
| crates/whirlpool-node.md | Per-crate contract/reference details | 146 |

## Reading guide for downstream agents

1. Load Tier 1 completely before proposing architecture or code changes.
2. Pull Tier 2 selectively based on task focus (`DOMAINS.md` for contracts, `TESTS.md` for validation, `WORKSPACE.md` for wiring/build, `BLOCKERS.md` for constraints).
3. Use Tier 3 only for targeted deep dives, provenance checks, and crate-specific implementation details.
4. For implementation planning, map each planned change to one Tier 1 flow and one crate reference (`crates/*.md`) before editing code.
