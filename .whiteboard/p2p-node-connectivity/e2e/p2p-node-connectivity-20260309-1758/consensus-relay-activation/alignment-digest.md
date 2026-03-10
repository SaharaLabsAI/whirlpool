# Alignment Digest

- Sub-Intent: C — Consensus Relay Activation
- Objective: Make the simplex relay broadcast functional so multi-node consensus can exchange proposed block payloads over P2P.
- Primary crates: `consensus-simplex` for relay implementation, with possible transport updates in `p2p` and `p2p-commonware` if a dedicated payload channel is required.
- Scope boundary: Do not modify vendor code. Focus on the application-level relay adapter in `consensus-simplex`. A new channel constant and corresponding registration in the P2P layer are in scope only if needed to carry payload distribution traffic.
- Dependencies: Sub-Intent A (P2P provider) complete; Sub-Intent B (NodeConfig) complete.
- Success criteria:
  - `Relay::broadcast` sends the payload associated with the digest to peers.
  - Receiving nodes store inbound payloads for later consensus verification.
  - `Automaton::verify` can access relayed payloads by digest.
  - Existing single-node tests continue to pass.
  - A new multi-node relay test validates end-to-end payload round-trip behavior.
