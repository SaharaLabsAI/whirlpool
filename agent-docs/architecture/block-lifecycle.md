# Block Lifecycle & Data Flow

This document maps block flow across consensus interface, adapter implementation, and node application.

## Layer Model
- Layer 1 (`consensus`): trait interfaces via `consensus::traits::*`.
- Layer 2 (`consensus-simplex`): adapter implementation using `consensus_simplex::traits::CommonwareBlock` + engine wiring.
- Layer 3 (node crates): concrete block/app logic and runtime startup.

## Proposal
1. Vendor simplex triggers propose callback.
2. `AppAdapter` extracts parent context.
3. Calls `consensus::traits::ConsensusApp::propose`.
4. Application returns block for broadcast.

## Verification
1. Vendor simplex triggers verify callback with ancestry.
2. `AppAdapter` forwards to `consensus::traits::ConsensusApp::verify`.
3. Application validates parent/height rules.

## Finalization
1. Vendor emits finalized update.
2. Adapter maps to `ConsensusEvent::Finalized`.
3. Calls `consensus::traits::EventSink::handle`.
4. Sink updates finalized-height observability state.

## Cross-Crate Mapping
| Interface crate | Adapter crate | Node/app crate |
|---|---|---|
| `consensus::traits::Block` | `consensus_simplex::traits::CommonwareBlock` | concrete block type |
| `consensus::traits::ConsensusApp` | `AppAdapter` | concrete app type |
| `consensus::traits::EventSink` | adapter event bridge | concrete sink type |
| `consensus::traits::ConsensusEngine` | `CommonwareEngine` | engine startup caller |
