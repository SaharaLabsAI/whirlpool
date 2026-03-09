# Risk Assessment — P2P Node Connectivity

## Iteration 1

### Identified Risks

| ID | Risk | Severity | Status | Resolution |
|----|------|----------|--------|------------|
| R-1 | Channel metadata bug (hard-coded Channel(0)) may cause message misrouting in consensus | High | Resolved by REQ-3 | Fix CommonwareReceiver to extract real channel from Commonware stream context |
| R-2 | Validator seeding gap means nodes can't learn about each other even with bootstrap peers | High | Resolved by REQ-1 | Call oracle_handle.update_validators with known validators at startup |
| R-3 | No existing CLI framework detected — unclear if clap/structopt already in deps | Medium | Accepted | Will check Cargo.toml deps; if no framework present, use clap (standard Rust) |
| R-4 | Relay no-op replacement scope may be larger than expected (consensus-simplex internals) | Medium | Accepted | Exploration shows relay/mailbox are well-isolated; REQ-6 is feasible |
| R-5 | Commonware vendor API changes could break p2p-commonware wrapper | Low | Accepted | Vendor is pinned via git submodule; no changes to vendor code |
| R-6 | NAT traversal not in scope — may limit real-world connectivity | Low | Accepted | Explicitly a non-goal; can be added later as separate intent |

### Summary
- Risks resolved: 2 (R-1, R-2 — addressed directly by requirements)
- Risks accepted: 4 (R-3 through R-6 — low/medium, mitigated)
- Blockers: 0
- Scope expansions: 0
