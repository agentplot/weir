//! Cadence: whether a scheduled thing is due (59), caught up once after a
//! machine has slept (60), read from a five-field cron schedule in the
//! machine's local time (C.5), with the clock as its one seam (83 to 85).
//!
//! The caller keeps the last run of each thing; this crate keeps nothing.
#![deny(missing_docs)]

mod schedule;

use jiff::{Timestamp, Zoned};

pub use schedule::{Field, Reason, Schedule, ScheduleError};

/// Whether a thing is due (59, 60), from its schedule, its last run (none
/// when it has never run) and now, the schedule read in now's time zone
/// (C.5).
pub fn is_due(schedule: &Schedule, last_run: Option<Timestamp>, now: &Zoned) -> bool {
    let Some(last_run) = last_run else {
        return true;
    };
    if last_run >= now.timestamp() {
        return false;
    }
    schedule
        .first_after(last_run, now.time_zone())
        .is_some_and(|named| named.timestamp() <= now.timestamp())
}
