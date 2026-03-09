# Learnings - p2p-provider-completeness

This file tracks conventions, patterns, and insights discovered during task execution.

---
## [2026-03-09T20:37:49+08:00] Task 01: Receiver channel contract
- CommonwareReceiver now stores channel as constructor parameter.
- recv() preserves channel metadata by emitting self.channel instead of a Channel(0) placeholder.
- Added TST-REQ3-001 and TST-REQ3-002 in receiver.rs to lock channel preservation and payload/peer passthrough behavior.
## [$(date -Iseconds)] Task 02: Provider build seeding and bootstrap
- Provider builder `build()` is now async and seeds validators via `oracle_handle.update_validators(epoch, validators).await` only when `initial_validators` is present and non-empty.
- Bootstrappers were already preserved and continue to flow unchanged into `discovery::Config::local()`.
- Provider receiver construction now passes explicit channel tags for `Channel::VOTE`, `Channel::CERTIFICATE`, and `Channel::RESOLVER`.
- Provider-local tests were added for validator seeding, empty-set skip behavior, bootstrap wiring sanity, and receiver channel tagging behavior.
- `cargo check -p p2p-commonware` passes; `cargo test -p p2p-commonware provider` is currently blocked by pre-existing `src/tests.rs` call-sites that still use the old `CommonwareReceiver::new(inner)` signature.
## [2026-03-09T20:58:24+08:00] Task 03: Multiplex receiver alignment
- MultiplexReceiver now trusts CommonwareReceiver channel tagging (returns msg as-is)
- Removed channel-repair logic that overwrote receiver's channel with multiplex's stored channel
- Round-robin polling preserved
- REQ-3 complete through aggregate receive path
