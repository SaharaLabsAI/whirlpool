//! Block-pipeline-owned invariant map.
//!
//! The block pipeline owns execution timing and app-local verification
//! obligations for proposal and verification. This module documents those
//! obligations and exposes only tiny pure predicates that do not hide the
//! propose/verify flow.
//!
//! Invariants that stay at their owner call sites:
//! - proposed blocks must be tied to the parent id computed from the parent;
//! - verified blocks must carry the expected parent id and transaction root;
//! - verified blocks must match the protocol-derived base fee, state root,
//!   receipts root, and gas used computed by the execution lane;
//! - proposal filters reserved epoch namespace transactions while verification
//!   rejects blocks that contain them;
//! - epoch-boundary system-call sequencing is an app call-site invariant, while
//!   epoch state/effect semantics remain owned by `evm-precompiles`;
//! - DKG header proposal/verification is an app call-site invariant, while DKG
//!   payload and activation semantics remain owned by `validators-dkg`.
//!
//! Accounting meaning remains owned by `crate::block_pipeline::accounting`; the
//! predicates below only express pure input-shape facts used by that owner.

pub mod accounting;
