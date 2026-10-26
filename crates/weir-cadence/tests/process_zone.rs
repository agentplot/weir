//! Cadence in processes of their own time zones: the same three values give
//! the same answer whatever the process's zone (59), and the machine's now
//! is in the machine's local zone (83, C.5). Each case runs this test binary
//! again with `TZ` set, so no test shares an environment variable with
//! another thread.

mod common;

use std::process::Command;

use common::{instant, schedule, zone};
use weir_cadence::{is_due, machine_now};

/// Run by the cases below in a process of its own, with `TZ` set: prints
/// the process's zone and its answers to fixed schedules, last runs and
/// nows.
#[test]
#[ignore = "run by the cases below with TZ set"]
fn report() {
    let daily = schedule("0 9 * * *");
    let hourly = schedule("0 * * * *");
    let last_run = Some(instant("2026-11-02T17:00Z"));
    let now = instant("2026-11-03T14:30Z");
    let answers = [
        is_due(&daily, last_run, &now.to_zoned(zone("Europe/Berlin"))),
        is_due(&daily, last_run, &now.to_zoned(zone("America/Los_Angeles"))),
        is_due(&hourly, Some(now), &now.to_zoned(zone("Asia/Tokyo"))),
        is_due(&hourly, None, &now.to_zoned(zone("Asia/Tokyo"))),
    ];
    let zone = machine_now().time_zone().iana_name().map(str::to_string);
    println!("zone: {}", zone.as_deref().unwrap_or("none"));
    println!("answers: {answers:?}");
}

/// What `report` printed in a process whose `TZ` is `tz`: its zone and its
/// answers.
fn reported(tz: &str) -> (String, String) {
    let exe = std::env::current_exe().expect("this test binary");
    let output = Command::new(exe)
        .args(["report", "--exact", "--ignored", "--nocapture"])
        .args(["--test-threads", "1"])
        .env("TZ", tz)
        .output()
        .expect("the test binary runs again");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "TZ={tz}: {stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let line = |label: &str| {
        stdout
            .lines()
            .find_map(|line| line.split_once(label).map(|(_, rest)| rest))
            .unwrap_or_else(|| panic!("TZ={tz}: no {label:?} in {stdout}"))
            .to_string()
    };
    (line("zone: "), line("answers: "))
}

#[test]
fn two_machines_given_the_same_three() {
    let (tokyo_zone, tokyo) = reported("Asia/Tokyo");
    let (los_angeles_zone, los_angeles) = reported("America/Los_Angeles");
    assert_eq!(tokyo_zone, "Asia/Tokyo");
    assert_eq!(los_angeles_zone, "America/Los_Angeles");
    assert_eq!(tokyo, los_angeles);
    assert_eq!(tokyo, "[true, false, false, true]");
}

#[test]
fn the_machines_now() {
    let (zone, _) = reported("Australia/Adelaide");
    assert_eq!(zone, "Australia/Adelaide");
}
