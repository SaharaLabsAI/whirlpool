# MANIFEST

## Inputs
- User intent: wire reth JSON-RPC into rpc-eth using adapter types
- Prior exploration: reth RPC crate map, whirlpool backend trait map, test patterns
- Alignment pre-approval from user

## Outputs
- scratch/agent/requirements.md — REQ-1..REQ-7
- scratch/agent/tests.md — TST-1..TST-12 (QA baseline)
- scratch/risk-assessment.md — 3 accepted risks
- review/alignment-digest.md — approved alignment
- e2e-state.md — phase=design
- SKILL_DIGEST.md — grounded facts from align phase

## Scope Notes
- No split required
- Read-only: vendor reth RPC crates
- Modify: crates/rpc-eth (primary), crates/whirlpool-node (integration)
