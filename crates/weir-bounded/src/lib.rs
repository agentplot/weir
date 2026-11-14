//! A call to a remote run to the family's limit for a try, or to its
//! caller's halt, of the statement in boomdrivedev/blueprints
//! `design/weir/requirements.md` (69; C.3; record 0008 as amended
//! 2026-11-12; core.53).
//!
//! A try runs in a process group of its own and is waited on no longer
//! than its limit or its caller's [`Halt`], whichever comes first. At the
//! first of them its group is asked to end, given [`GRACE`], then killed,
//! so nothing it started outlives the call, and the call is said stopped
//! by that one: at its limit, or halted. A try begun with its halt set
//! runs no process. What it prints is written to unnamed temporary files
//! and read once it has ended: the crate starts no thread, and holds no
//! halt in a static (core.1). The limit, the halt, the group's ending and
//! a stopped call's words are written here alone, and every call weir
//! makes to a remote takes them.

#![deny(missing_docs)]

use std::time::Duration;

mod group;
mod halt;
mod run;

pub use group::end_group;
pub use halt::Halt;
pub use run::{output, output_kept, Ended, Stopped};

/// The most a try of a call to a remote may take: the family's limit, one
/// value no setting changes (C.3; record 0008).
pub const TRY_FOR: Duration = Duration::from_secs(30);

/// How long a group asked to end at its limit has before it is killed:
/// git, asked, removes the lock files it holds in the repository.
pub const GRACE: Duration = Duration::from_secs(2);

/// A limit in the words a person reads: whole seconds as `30s`, anything
/// else in milliseconds, as `300ms`.
pub fn span(limit: Duration) -> String {
    if limit >= Duration::from_secs(1) && limit.subsec_nanos() == 0 {
        format!("{}s", limit.as_secs())
    } else {
        format!("{}ms", limit.as_millis())
    }
}

/// A call's words where its limit came first, naming its call, its remote
/// and its limit.
pub fn stopped(call: &str, remote: &str, limit: Duration) -> String {
    format!(
        "{call} with {remote} was still going after {}, the most a try may take, and was stopped",
        span(limit)
    )
}

/// A call's words where its caller's halt came first, naming its call and
/// its remote, and no limit, since the call was not still going at one.
pub fn halted(call: &str, remote: &str) -> String {
    format!("{call} with {remote} was stopped as its caller asked")
}
