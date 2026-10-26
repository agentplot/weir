//! The shared-branch write protocol of the statement in agentplot/blueprints
//! `design/weir/requirements.md`: how every writer of a shared branch reads
//! it and writes it, so that no write is lost and no two writes move the
//! branch from one tip (44 to 50, 78 to 80; C.3).
//!
//! A write is formed by its caller against the tip read, checked there,
//! made one commit through a temporary index touching no working copy, and
//! landed only while the branch still stands at that tip; a moved branch
//! has it formed again on the new tip, up to the writer's bound. A lock in
//! the git directory keeps one machine's writers in turn and never stands
//! in for the move from the tip read.
//!
//! The crate keeps nothing between calls: the branch, its cache, the bound,
//! the lock's stale age and the clock are the caller's to hand in. It runs
//! git and `recfix` for the write it was asked to make, each ended before
//! the call returns (69; C.3).

mod commit;
mod git;
pub mod keeper;
pub mod lock;
mod recfix;
pub mod write;

pub use keeper::{read, Branch, Failure, Files, Place, Step, Tip, TipError};
pub use lock::{Broken, LOCK};
pub use write::{write, Ask, Landed, Nothing, WriteError, Written, BOUND};
