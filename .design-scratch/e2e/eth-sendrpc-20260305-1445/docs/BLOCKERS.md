# BLOCKERS

| ID | Type | Status | Summary | Evidence | Resolution |
|----|------|--------|---------|----------|------------|
| BLK-001 | decision-gap | resolved | Receipt persistence source was initially unclear because no dedicated receipt store exists in current trait surfaces. | `crates/app-evm/src/executor.rs::EvmApplication::propose`, `crates/state/src/traits.rs::StateDb` | Constrained v1 design to node-local receipt index for required transfer polling semantics; documented extension path for richer receipt persistence. |

## Open blocker check
- No active `scope-expansion` blockers.
- No active `decision-gap` blockers.
- No active `information-gap` blockers that block this design set.
