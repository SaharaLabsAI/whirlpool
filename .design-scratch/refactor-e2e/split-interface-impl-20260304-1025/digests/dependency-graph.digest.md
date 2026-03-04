# dependency-graph.digest

- **Grounded**: Structural layering is acyclic across in-scope crates: foundational interfaces (`consensus`, `p2p`, `state`) feed into `app`, then implementation/adapters (`consensus-simplex`, `p2p-commonware`, `app-evm`), then consumers (`whirlpool-node`, `whirlpool-node-simple`).
- **Grounded**: `consensus` and `p2p` expose `mock`/cfg-driven test surfaces consumed downstream (`consensus-simplex` depends on `p2p` with feature `mock`), so trait-file moves must preserve cfg/export behavior.
- **Grounded**: `state` is a leaf with reverse deps `app-evm` and `whirlpool-node`, making `state::traits::StateDb` a safe introduction point if no reverse references are added.
- **Grounded**: `p2p-commonware` and `consensus-simplex` are implementation-heavy adapters and should remain downstream of trait definitions to prevent cycles.
- **[PROPOSED]**: Sequence refactor in dependency order (foundation first, adapters second, top-level consumers last).
- **UNKNOWN**: Full cfg-interaction edge cases from previously referenced but unavailable background task artifact.
- **BLOCKER**: None detected; dependency graph currently shows no circularity.
