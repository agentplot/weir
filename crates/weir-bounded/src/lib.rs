//! A call to a remote run to the family's limit for a try, of the statement
//! in boomdrivedev/blueprints `design/weir/requirements.md` (69; C.3; record
//! 0008; core.53).
//!
//! A try runs in a process group of its own and is waited on no longer
//! than its limit. At the limit its group is asked to end, given
//! [`GRACE`], then killed, so nothing it started outlives the call, and
//! the call is said stopped. What it prints is written to unnamed
//! temporary files and read once it has ended: the crate starts no thread
//! (core.1). The limit, the group's ending and a stopped call's words are
//! written here alone, and every call weir makes to a remote takes them.

#![deny(missing_docs)]

use std::time::Duration;

mod group;
mod halt;
mod run;

pub use group::end_group;
pub use halt::Halt;
pub use run::{output, output_kept, Ended};

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

/// A stopped call's words, naming its call, its remote and its limit.
pub fn stopped(call: &str, remote: &str, limit: Duration) -> String {
    format!(
        "{call} with {remote} was still going after {}, the most a try may take, and was stopped",
        span(limit)
    )
}
