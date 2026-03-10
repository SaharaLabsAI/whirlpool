# Risk Assessment

## HIGH
### Vendor trait contract risk
- Risk: The simplex `Relay` trait receives only a `Digest`, not the payload bytes.
- Impact: The relay must rely on a correct digest-to-payload cache populated earlier by the automaton path. If that mapping is missing, stale, or inconsistent, proposal distribution fails and consensus can stall.
- Mitigation:
  - Treat mailbox-side block storage as the required source of truth for digest-to-payload lookup.
  - Preserve the ordering assumption that payload capture happens before relay broadcast is attempted.
  - Validate the relay path with a multi-node round-trip test that proves a proposed payload can be looked up and sent by digest.

## MEDIUM
### Channel allocation risk
- Risk: The existing vote, cert, and resolver channels are already dedicated to vendor simplex traffic.
- Impact: Payload distribution may require a fourth channel or another clearly separate mechanism, which introduces registration and routing changes.
- Mitigation:
  - Keep vendor-managed channels unchanged.
  - Add a dedicated payload distribution path only at the application transport layer if required.
  - Verify channel constant registration remains aligned across crates before implementation proceeds.

### Payload availability timing risk
- Risk: `broadcast()` is invoked after proposal creation, and it depends on the payload already being recoverable by digest.
- Impact: If the payload is not cached before the engine calls `broadcast(digest)`, the relay cannot send the proposal body and downstream validators cannot verify it.
- Mitigation:
  - Preserve the expected flow: `propose()` yields the proposal result, the payload is stored, then `broadcast(digest)` resolves that digest back to payload.
  - Confirm the adapter ordering during implementation and cover it with a targeted test.

### Cross-crate boundary risk
- Risk: The required changes likely touch both `consensus-simplex` and transport-layer registration in `p2p` or `p2p-commonware`.
- Impact: Incomplete alignment across crate boundaries could produce a relay path that compiles in one crate but is not routable end to end.
- Mitigation:
  - Keep the primary logic in `consensus-simplex`.
  - Limit transport-layer changes to channel constant definition and registration only if the existing channel set cannot carry payload relay traffic.
  - Verify any new channel constant is consistently defined and consumed across the relevant crates.

## LOW
### Single-node backward compatibility risk
- Risk: Relay activation for multi-node mode could accidentally disrupt existing single-node behavior.
- Impact: Current local-only operation may regress even though it does not depend on peer payload exchange.
- Mitigation:
  - Preserve behavior when no peers are present.
  - Keep existing single-node tests in the validation set.
  - Ensure relay enablement does not require remote peers to make local consensus progress.
