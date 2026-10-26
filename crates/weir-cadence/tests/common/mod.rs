//! Fixed inputs for the tests: zones from jiff's bundled database, so a
//! host's own copy cannot change a result.

#![allow(dead_code)]

use jiff::civil::DateTime;
use jiff::tz::{TimeZone, TimeZoneDatabase};
use jiff::{SignedDuration, Timestamp, Zoned};
use weir_cadence::{is_due, Schedule};

/// The zone `name`, from the bundled database.
pub fn zone(name: &str) -> TimeZone {
    TimeZoneDatabase::bundled()
        .get(name)
        .unwrap_or_else(|error| panic!("{name}: {error}"))
}

/// The civil time `civil`, such as `2026-11-03T09:00`, in the zone `name`.
pub fn local(civil: &str, name: &str) -> Zoned {
    let civil: DateTime = civil.parse().expect(civil);
    civil.to_zoned(zone(name)).expect(name)
}

/// The instant `text`, written with its offset.
pub fn instant(text: &str) -> Timestamp {
    text.parse().expect(text)
}

/// A schedule the test expects to read.
pub fn schedule(text: &str) -> Schedule {
    Schedule::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// Whether `schedule` names the whole second `at`: a time it names lies in
/// the second up to and including `at`.
pub fn names(schedule: &Schedule, at: &Zoned) -> bool {
    let before = at.timestamp() - SignedDuration::from_secs(1);
    is_due(schedule, Some(before), at)
}

/// Every minute from `from` for `minutes` minutes on which `names` and
/// `expected` disagree, each with what `names` said.
pub fn disagreements(
    schedule: &Schedule,
    from: &Zoned,
    minutes: i64,
    expected: impl Fn(&Zoned) -> bool,
) -> Vec<(String, bool)> {
    (0..minutes)
        .map(|minute| from + SignedDuration::from_mins(minute))
        .filter_map(|at| {
            let named = names(schedule, &at);
            (named != expected(&at)).then(|| (at.to_string(), named))
        })
        .collect()
}
