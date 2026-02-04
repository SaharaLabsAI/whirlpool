# `consensus`

**Purpose**: decide canonical/finalized blocks.

Owns: vote/finality rules, consensus state machine/driver, fork choice (if any).

Inputs: candidate blocks (headers + execution results) + peer votes/messages.

Outputs: finalized head + consensus events.

Depends on: `types`, `core` (traits), `storage`.
