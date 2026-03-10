# SUMMARY — Split Interface From Implementation

This design set is finalize-ready for a structural interface/implementation split across seven crates: `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, and `app-evm`.

The package preserves behavior and limits change to module boundaries. Trait contracts are normalized into explicit interface modules, concrete types are moved into implementation modules, and compatibility re-exports are used to keep downstream crates compiling during transition.

The migration strategy is explicitly ordered and dependency-safe: foundation crates first (`consensus`, `state`, `p2p`), then app abstraction and EVM boundary (`app`, `app-evm`), then highest-coupling adapter moves (`consensus-simplex`, `p2p-commonware`) and consumer import cleanup. Final compatibility-export removal is deferred to the last step after canonical-path adoption.

`MIGRATION.md` provides 9 bounded steps with prerequisites, verification commands, and rollback instructions. This enforces incremental compilability and avoids future-step dependencies. `TESTS.md` mirrors that sequence with step-aligned expected breakage (`TB-*`), additive safeguards (`TN-*`), and per-step verification commands.

Safety gate review is clean: no circular dependency introduction is identified, public API path churn has compatibility handling, and test coverage alignment spans all migration steps (1-9). Per-crate `CHANGES.md` files are present for every in-scope crate and align with both strategy and migration ordering.

Recommendation: proceed with implementation using the documented wave order and verification spine.
