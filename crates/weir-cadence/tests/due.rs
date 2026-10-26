//! Whether a thing is due (59), caught up once (60), on daylight-saving
//! days (C.5), and through the clock (83 to 85).

mod common;

use std::cell::RefCell;

use common::{instant, local, schedule, zone};
use jiff::{Timestamp, Zoned};
use weir_cadence::{due, is_due, Clock};

const NEW_YORK: &str = "America/New_York";

fn new_york(civil: &str) -> Zoned {
    local(civil, NEW_YORK)
}

fn ran(civil: &str) -> Option<Timestamp> {
    Some(new_york(civil).timestamp())
}

#[test]
fn first_tick_of_a_new_schedule() {
    let daily = schedule("0 9 * * *");
    assert!(is_due(&daily, None, &new_york("2026-11-03T08:00")));
}

#[test]
fn before_the_next_named_time() {
    let hourly = schedule("0 * * * *");
    assert!(!is_due(
        &hourly,
        ran("2026-11-03T09:00:00"),
        &new_york("2026-11-03T09:59:59")
    ));
}

#[test]
fn at_the_next_named_time() {
    let hourly = schedule("0 * * * *");
    assert!(is_due(
        &hourly,
        ran("2026-11-03T09:00:00"),
        &new_york("2026-11-03T10:00:00")
    ));
}

#[test]
fn a_run_that_began_on_the_named_time() {
    let hourly = schedule("0 * * * *");
    assert!(!is_due(
        &hourly,
        ran("2026-11-03T10:00:00"),
        &new_york("2026-11-03T10:00:30")
    ));
}

#[test]
fn a_run_that_began_a_fraction_before_the_named_time() {
    let hourly = schedule("0 * * * *");
    assert!(is_due(
        &hourly,
        ran("2026-11-03T09:59:59.700"),
        &new_york("2026-11-03T10:00:00.000")
    ));
}

#[test]
fn a_run_that_began_a_fraction_after_the_named_time() {
    let hourly = schedule("0 * * * *");
    assert!(!is_due(
        &hourly,
        ran("2026-11-03T10:00:00.300"),
        &new_york("2026-11-03T10:59:59.999")
    ));
}

#[test]
fn a_last_run_later_than_now() {
    let hourly = schedule("0 * * * *");
    assert!(!is_due(
        &hourly,
        ran("2026-11-03T11:00:00"),
        &new_york("2026-11-03T10:30:00")
    ));
}

#[test]
fn the_same_instant_in_two_zones() {
    let daily = schedule("0 9 * * *");
    let last_run = Some(instant("2026-11-02T17:00Z"));
    let now = instant("2026-11-03T14:30Z");
    let berlin = now.to_zoned(zone("Europe/Berlin"));
    let los_angeles = now.to_zoned(zone("America/Los_Angeles"));
    assert_eq!(berlin.hour(), 15);
    assert_eq!(los_angeles.hour(), 6);
    assert!(is_due(&daily, last_run, &berlin));
    assert!(!is_due(&daily, last_run, &los_angeles));
}

#[test]
fn three_hourly_times_slept_through() {
    let hourly = schedule("0 * * * *");
    // Asleep from just after the 06:00 run until 09:20: one answer for the
    // 07:00, 08:00 and 09:00 missed.
    assert!(is_due(
        &hourly,
        ran("2026-11-03T06:00"),
        &new_york("2026-11-03T09:20")
    ));
    // The catch-up runs at 09:20, and is the last run from then.
    let last_run = ran("2026-11-03T09:20");
    assert!(!is_due(&hourly, last_run, &new_york("2026-11-03T09:21")));
    assert!(!is_due(&hourly, last_run, &new_york("2026-11-03T09:59")));
    assert!(is_due(&hourly, last_run, &new_york("2026-11-03T10:00")));
}

#[test]
fn a_time_the_clock_skips() {
    let daily = schedule("30 2 * * *");
    let new_york = zone(NEW_YORK);
    let last_run = Some(instant("2026-03-07T02:30-05:00"));
    // 2026-03-08 skips from 02:00 to 03:00: 03:00 is the first minute after
    // the skip, and 02:59 the one before it.
    let after = instant("2026-03-08T03:00-04:00").to_zoned(new_york.clone());
    let before = instant("2026-03-08T01:59-05:00").to_zoned(new_york.clone());
    assert!(is_due(&daily, last_run, &after));
    assert!(!is_due(&daily, last_run, &before));
    // Named once that day: the run at 03:00 is not due again until the next
    // day's 02:30.
    let caught_up = Some(after.timestamp());
    let evening = instant("2026-03-08T23:59-04:00").to_zoned(new_york.clone());
    let next = instant("2026-03-09T02:30-04:00").to_zoned(new_york);
    assert!(!is_due(&daily, caught_up, &evening));
    assert!(is_due(&daily, caught_up, &next));
}

#[test]
fn a_time_the_clock_repeats() {
    let daily = schedule("30 1 * * *");
    let new_york = zone(NEW_YORK);
    // 2026-11-01 falls back from 02:00 to 01:00, so 01:30 comes twice.
    let first_pass = instant("2026-11-01T01:30-04:00");
    let second_pass = instant("2026-11-01T01:45-05:00").to_zoned(new_york.clone());
    assert!(!is_due(&daily, Some(first_pass), &second_pass));
    // Named at the first pass: a run from the day before is due by then.
    let day_before = Some(instant("2026-10-31T01:30-04:00"));
    let at_first_pass = first_pass.to_zoned(new_york.clone());
    assert!(is_due(&daily, day_before, &at_first_pass));
    // And not again on the second pass's 01:30.
    let second_half_hour = instant("2026-11-01T01:30-05:00").to_zoned(new_york);
    assert!(!is_due(&daily, Some(first_pass), &second_half_hour));
}

/// What a clock was asked.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    Now,
    LastRun(String),
}

/// A clock that answers from fixed values and records what it is asked.
struct Recording {
    now: Zoned,
    last_run: Option<Timestamp>,
    asked: RefCell<Vec<Asked>>,
}

impl Clock for Recording {
    type Thing = str;

    fn now(&self) -> Zoned {
        self.asked.borrow_mut().push(Asked::Now);
        self.now.clone()
    }

    fn last_run(&self, thing: &str) -> Option<Timestamp> {
        self.asked
            .borrow_mut()
            .push(Asked::LastRun(thing.to_string()));
        self.last_run
    }
}

#[test]
fn the_clock_is_asked_for_two_things() {
    let hourly = schedule("0 * * * *");
    for (last_run, now) in [
        (None, "2026-11-03T09:20"),
        (ran("2026-11-03T06:00"), "2026-11-03T09:20"),
        (ran("2026-11-03T09:20"), "2026-11-03T09:21"),
    ] {
        let clock = Recording {
            now: new_york(now),
            last_run,
            asked: RefCell::new(Vec::new()),
        };
        let answer = due(&hourly, "the backup", &clock);
        assert_eq!(answer, is_due(&hourly, last_run, &clock.now));
        let mut asked = clock.asked.into_inner();
        asked.sort_by_key(|asked| matches!(asked, Asked::LastRun(_)));
        assert_eq!(
            asked,
            vec![Asked::Now, Asked::LastRun("the backup".to_string())]
        );
    }
}
