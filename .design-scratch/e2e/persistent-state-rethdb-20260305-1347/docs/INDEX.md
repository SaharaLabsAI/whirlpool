# Design Docs Index

## Overview

This design doc set covers persistent state integration for Whirlpool using `reth-db` (MDBX). Total documentation: **2,357 lines** across 13 files.

**Design iteration:** `persistent-state-rethdb-20260305-1347`  
**Workspace root:** `/home/dev/sahara/web3/agent/playground/whirlpool`

---

## File Inventory

### Tier 1: Foundation Documents (Always Load)

| File | Lines | Purpose |
|------|-------|---------|
| `INTENT.md` | 34 | Parsed objective, concrete requirements, and scope boundaries |
| `SHARED_CONTEXT.md` | 134 | Workspace architecture, type system, dependencies, and reth-db API patterns |
| `BLOCKERS.md` | 12 | Hard and soft blockers affecting downstream design phases |

**Total Tier 1:** 180 lines

### Tier 2: Strategic Design (Load on Demand)

| File | Lines | Purpose |
|------|-------|---------|
| `STRATEGY.md` | 389 | Crate allocation, module boundaries, trait design, concurrency model, table mapping, implementation phases |
| `CRATES.md` | 195 | Per-crate design contracts: `state-reth` (new), `state` (modified), `whirlpool-node` (modified) |
| `WORKSPACE.md` | 146 | Workspace membership, integration topology, build order, feature flags |
| `DOMAINS.md` | 285 | Domain definitions: Persistent Storage, State Interface, State Root, Node Wiring |

**Total Tier 2:** 1,015 lines

### Tier 3: Execution Design (Load for Implementation Planning)

| File | Lines | Purpose |
|------|-------|---------|
| `FLOWS.md` | 255 | Six architecture-level flows with caller/callee chains, data contracts, error paths |
| `TESTS.md` | 226 | Test contracts: 46 test cases (unit, integration, property tests) mapped to success criteria |

**Total Tier 3:** 481 lines

### Tier 4: Supporting Context (Reference Only)

| File | Lines | Purpose |
|------|-------|---------|
| `EXPLORATION.md` | 151 | Pre-digested exploration findings (architecture, types, dependencies, reth-db patterns) |

**Total Tier 4:** 151 lines

### Tier 5: Per-Crate Contracts (Load by Crate)

| File | Lines | Purpose |
|------|-------|---------|
| `crates/state-reth/README.md` | 255 | `state-reth` public API, table mapping, state root contract, error handling, concurrency guarantees |
| `crates/state/README.md` | 134 | `state` trait fallibility migration, error handling, migration notes |
| `crates/whirlpool-node/README.md` | 141 | Node wiring changes, initialization sequence, startup error handling |

**Total Tier 5:** 530 lines

---

## Reading Guide

### For First-Time Readers

**Minimum viable context (10 minutes):**
1. `INTENT.md` — understand the objective and scope
2. `BLOCKERS.md` — identify what's still unresolved
3. `STRATEGY.md` sections: Crate Allocation, Trait Design, Table Mapping

### For Implementers

**Implementation planning (30 minutes):**
1. Tier 1: `INTENT.md`, `SHARED_CONTEXT.md`, `BLOCKERS.md`
2. Tier 2: `STRATEGY.md`, `CRATES.md`
3. Tier 3: `FLOWS.md` (read flows relevant to your task)
4. Tier 5: Read README for your target crate(s)

**Critical sections by role:**
- **`state-reth` implementer:** `STRATEGY.md` (Module Boundaries, Table Mapping, State Root Strategy), `crates/state-reth/README.md`, `FLOWS.md` (Flows 3, 4, 5)
- **`state` trait migrator:** `STRATEGY.md` (Trait Design), `crates/state/README.md`, `CRATES.md` (state section)
- **`whirlpool-node` integrator:** `STRATEGY.md` (Implementation Phases), `crates/whirlpool-node/README.md`, `FLOWS.md` (Flow 1, 2)
- **Test author:** `TESTS.md` (full read), `FLOWS.md` (error paths)

### For Reviewers

**Architecture review (45 minutes):**
1. Tier 1: All foundation docs
2. `STRATEGY.md`: Trait Design, Concurrency Model, Error Handling Strategy, Risk Mitigation
3. `DOMAINS.md`: Cross-Domain Boundaries, Wiring Risks Summary
4. `FLOWS.md`: Error Propagation Flow (Flow 6)
5. `TESTS.md`: Success Criteria Summary

**Design coherence check:**
- Verify all blockers in `BLOCKERS.md` are addressed in `STRATEGY.md` or delegated with rationale
- Verify `CRATES.md` crate contracts match `STRATEGY.md` allocations
- Verify `FLOWS.md` caller/callee chains match `DOMAINS.md` wiring table
- Verify `TESTS.md` test cases cover `INTENT.md` success criteria

### For Project Managers

**Executive overview (15 minutes):**
1. `INTENT.md` — objective and requirements
2. `BLOCKERS.md` — hard vs. soft blockers
3. `STRATEGY.md` sections: Crate Allocation, Implementation Phases, Acceptance Criteria
4. `TESTS.md` section: Success Criteria Summary

---

## Key Cross-References

### Blocker Tracking

| Blocker ID | Status | Resolution Location |
|------------|--------|---------------------|
| BLK-001 | Hard | `STRATEGY.md` (Trait Design), `crates/state/README.md` (Blocker Resolution Notes) |
| BLK-002 | Hard | `STRATEGY.md` (State Root Strategy), `crates/state-reth/README.md` (State Root Contract) |
| BLK-003 | Hard | `STRATEGY.md` (Risk Mitigation), `crates/whirlpool-node/README.md` (Blocker Resolution Notes) |
| BLK-101 | Soft | `STRATEGY.md` (Table Mapping), `crates/state-reth/README.md` (Table Mapping Contract) |
| BLK-102 | Soft | `STRATEGY.md` (Error Handling Strategy), `crates/state-reth/README.md` (Error Handling Strategy) |
| BLK-103 | Soft | `STRATEGY.md` (Risk Mitigation), deferred to implementation |

### Flow-to-Domain Mapping

| Flow | Domains | Reference |
|------|---------|-----------|
| Flow 1: Database Initialization | D4 → D1 → D2 | `FLOWS.md` lines 16-54, `DOMAINS.md` Domain 4 |
| Flow 2: Genesis Bootstrap | D4 → D2 → D1 → D3 | `FLOWS.md` lines 57-93, `DOMAINS.md` Domain 3 |
| Flow 3: Transaction Execution (read) | D2 → D1 | `FLOWS.md` lines 96-143, `DOMAINS.md` Domain 2 |
| Flow 4: State Commit (write) | D2 → D1 → D3 | `FLOWS.md` lines 146-179, `DOMAINS.md` Domain 1 |
| Flow 5: State Root Computation | D2 → D3 → D1 | `FLOWS.md` lines 182-212, `DOMAINS.md` Domain 3 |
| Flow 6: Error Propagation | D1 → D2 → D4 | `FLOWS.md` lines 215-246, `DOMAINS.md` Cross-Domain Boundaries |

### Test-to-Flow Mapping

| Test Section | Flow Coverage | Priority |
|--------------|---------------|----------|
| `TC-SR-U001` through `TC-SR-U008` | Flow 3, Flow 4 | P0 |
| `TC-SR-U009`, `TC-SR-U010` | Flow 5 | P0 |
| `TC-SR-U011` through `TC-SR-U013` | Flow 3 (revm integration) | P0 |
| `TC-SR-I001` through `TC-SR-I004` | Flow 4 (durability + concurrency) | P0 |
| `TC-SR-I005`, `TC-SR-I006` | Flow 2 | P0 |
| `TC-WN-I001`, `TC-WN-I002` | Flow 1 | P0 |
| `TC-CC-I001` through `TC-CC-I003` | End-to-end flows | P0 |
| `TC-CC-I005`, `TC-CC-I006` | Flow 6 | P1 |

---

## Design Artifact Lineage

### Dependency Chain

```
INTENT.md (requirements)
  ↓
SHARED_CONTEXT.md (exploration findings)
  ↓
BLOCKERS.md (constraints)
  ↓
STRATEGY.md (design decisions)
  ↓
├── CRATES.md (per-crate contracts)
├── WORKSPACE.md (integration topology)
├── DOMAINS.md (domain boundaries)
├── FLOWS.md (execution flows)
└── TESTS.md (validation strategy)
  ↓
crates/*/README.md (implementation contracts)
```

### Grounded vs. Proposed Markers

- **[GROUNDED]**: Derived from existing code exploration or explicit user approval
- **[PROPOSED]**: Design decision requiring validation during implementation

**Grounding sources:**
- `SHARED_CONTEXT.md` — exploration findings from existing crates
- `EXPLORATION.md` — pre-digested architecture and API patterns
- `INTENT.md` — user-approved objective and requirements

---

## Metadata

- **Design doc set version:** persistent-state-rethdb-20260305-1347
- **Total lines:** 2,357
- **Total files:** 13 (10 top-level + 3 per-crate)
- **Design depth:** `module` (no full-system redesign)
- **Crates affected:** 3 (1 new: `state-reth`, 2 modified: `state`, `whirlpool-node`)
- **Domains:** 4 (Persistent Storage, State Interface, State Root, Node Wiring)
- **Flows:** 6 (init, genesis, read, write, state root, error propagation)
- **Test cases:** 46 (26 P0, 12 P1, 8 P2)
- **Hard blockers:** 3 (BLK-001, BLK-002, BLK-003)
- **Soft blockers:** 3 (BLK-101, BLK-102, BLK-103)

---

## Usage Notes

### When to Read This Index First

- You're new to this design doc set
- You need to find a specific topic quickly
- You're planning implementation work and need to load the right subset of docs

### When to Skip This Index

- You're already familiar with the design and need to dive into a specific document
- You're reviewing a specific pull request and have a direct file reference

### Document Loading Strategy

**For AI coding agents:**
1. Always load Tier 1 first (foundation context)
2. Load Tier 2 selectively based on task (use file purpose to decide)
3. Load Tier 5 per-crate READMEs when implementing specific crates
4. Use Tier 4 (EXPLORATION.md) only if you need to verify exploration findings

**For human readers:**
- Follow the "Reading Guide" section above based on your role
- Use "Key Cross-References" to navigate between related sections
- Check "Blocker Tracking" to understand what's still unresolved

---

## Change History

- **2026-03-05:** Initial INDEX.md generated for design iteration `persistent-state-rethdb-20260305-1347`
