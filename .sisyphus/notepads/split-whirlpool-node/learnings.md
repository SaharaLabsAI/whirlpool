# Learnings

## [2026-02-28T02:41:07Z] Session Start: split-whirlpool-node
Plan loaded. 6 tasks across 4 waves. Worktree: /home/dev/sahara/web3/agent/playground/whirlpool-split

## [2026-02-28T02:41:40Z] Task 2: Create whirlpool-node-simple
- Binary compiles: yes (exit code 0 after commonware-runtime feature removal)
- No EVM deps: confirmed (grep -E returned no matches)
- No cfg gates: confirmed (grep 'cfg.*feature' returned no matches)
- Dependencies: whirlpool-node (lib), consensus, consensus-simplex, p2p-commonware, commonware-cryptography, commonware-runtime, tokio, tracing
- Key fix: commonware-runtime does NOT have a "tokio" feature - must be path only (no features=[])
- Key learning: whirlpool-node-simple added to workspace members BEFORE attempting build (order matters)
- Bootstrap code: Successfully duplicated from whirlpool-node/src/main.rs lines 88-161
- All imports copied correctly - uses whirlpool_node::config for constants (VALIDATOR_SEED, NAMESPACE)
