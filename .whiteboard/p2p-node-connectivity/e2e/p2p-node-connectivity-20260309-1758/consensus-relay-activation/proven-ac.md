# Proven Acceptance Criteria (Sub-Intent C)

This document tracks the status of acceptance criteria for Sub-Intent C: Consensus Relay Activation.

| ID | Status | Requirement | Description |
|---|---|---|---|
| AC-C-1 | PENDING | REQ-7 | PAYLOAD channel constant (value=3) exists in p2p types |
| AC-C-2 | PENDING | REQ-7 | p2p-commonware registers PAYLOAD channel and exposes it in PerChannelNetwork |
| AC-C-3 | PENDING | REQ-6 | Mailbox::broadcast(digest) looks up payload from BlockStore and sends via PAYLOAD sender |
| AC-C-4 | PENDING | REQ-6 | Inbound payload receiver task stores received payloads in BlockStore |
| AC-C-5 | PENDING | REQ-8 | End-to-end: propose on node A → broadcast → node B receives → verify succeeds |
| AC-C-6 | PENDING | REQ-8 | Single-node backward compatibility: existing tests pass without peers |
| AC-C-7 | PENDING | REQ-7 | Channel constant alignment: p2p PAYLOAD=3 matches p2p-commonware registration |
