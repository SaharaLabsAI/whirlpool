//! Crate-private invariant review surface for `app-evm-execution`.
//!
//! This module is intentionally **not** public API: `lib.rs` wires it with
//! private `mod invariants;`, so child `pub` items are reachable only through
//! a crate-private parent path. It gives reviewers, fuzzing work, and future
//! formal-verification work one place to discover the
//! invariants owned by this crate, while keeping orchestration and semantic
//! authority in the existing owner modules.
//!
//! Ownership guardrails:
//! - `block_pipeline` owns proposal, verification, execution ordering, and
//!   lower-layer call-site timing.
//! - `post_handle` owns receipt staging and finalization lifecycle state.
//! - `validators-dkg` owns DKG payload and activation semantics.
//! - `evm-precompiles` owns epoch-boundary, community-pool, and post-block
//!   accounting write semantics.
//!
//! Invariant helpers here must stay tiny, pure, and behavior-preserving. Checks
//! that are entangled with orchestration, I/O, database access, or lower-crate
//! semantics should be named in docs here but remain implemented at their owner
//! call site.

pub mod block_pipeline;
pub mod post_handle;
