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

/// The clock (B.5): now (83) and a thing's last run (84), the two things
/// cadence asks of it (85).
pub trait Clock {
    /// A thing with a schedule (84), as the caller names it.
    type Thing: ?Sized;

    /// Now (83), in the time zone a schedule is read in.
    fn now(&self) -> Zoned;

    /// When `thing` last ran, or none when it never has (84).
    fn last_run(&self, thing: &Self::Thing) -> Option<Timestamp>;
}

/// Whether `thing` is due by `schedule`, asking `clock` for now and the
/// thing's last run once each (85), and answering as [`is_due`].
pub fn due<C: Clock>(schedule: &Schedule, thing: &C::Thing, clock: &C) -> bool {
    let now = clock.now();
    let last_run = clock.last_run(thing);
    is_due(schedule, last_run, &now)
}

/// The machine's now (83): the system clock, in the machine's local time
/// zone (C.5).
pub fn machine_now() -> Zoned {
    Zoned::now()
}
