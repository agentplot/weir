//! A schedule is five-field cron, and a schedule that is not is refused
//! (C.5).

mod common;

use common::{disagreements, instant, local, schedule};
use jiff::civil::Weekday;
use jiff::tz::TimeZone;
use jiff::Zoned;
use weir_cadence::{is_due, Field, Reason, Schedule, ScheduleError};

const WEEK: i64 = 7 * 24 * 60;

fn utc(civil: &str) -> Zoned {
    local(civil, "UTC")
}

fn nine(at: &Zoned) -> bool {
    at.hour() == 9 && at.minute() == 0
}

fn refused(text: &str) -> ScheduleError {
    match Schedule::parse(text) {
        Ok(read) => panic!("{text} was read: {read:?}"),
        Err(error) => error,
    }
}

fn field(field: Field, token: &str, reason: Reason) -> ScheduleError {
    ScheduleError::Field {
        field,
        token: token.to_string(),
        reason,
    }
}

#[test]
fn every_form_of_five_field_cron_is_read() {
    for text in [
        "* * * * *",
        "0 9 * * *",
        "5,35 0-23 1-31 1-12 0-7",
        "*/15 */2 */10 */3 */1",
        "0-30/10 9-17/4 1-15/7 1-11/2 1-5/2",
        "0 9 1,15 jan,Jul,DEC mon-fri",
        "0 9 * * SUN,sat",
        "  0   9  *  *  *  ",
        "00 09 01 01 00",
    ] {
        schedule(text);
    }
}

#[test]
fn lists_ranges_steps_and_names() {
    let read = schedule("*/15 9-17 * JAN,JUL mon-fri");
    let expected = |at: &Zoned| {
        matches!(at.month(), 1 | 7)
            && !matches!(at.weekday(), Weekday::Saturday | Weekday::Sunday)
            && (9..=17).contains(&at.hour())
            && at.minute() % 15 == 0
    };
    // A week each of January and July, and of February and December, where
    // it names nothing.
    for from in [
        "2026-01-03T00:00",
        "2026-07-29T00:00",
        "2026-02-02T00:00",
        "2026-12-27T00:00",
    ] {
        assert_eq!(
            disagreements(&read, &utc(from), WEEK, expected),
            vec![],
            "from {from}"
        );
    }
    let first = utc("2026-01-05T09:00");
    assert!(expected(&first) && expected(&utc("2026-01-05T17:45")));
    assert!(!expected(&utc("2026-01-05T18:00")));
}

#[test]
fn a_step_over_a_range() {
    let read = schedule("0 1-11/2 * * *");
    let expected = |at: &Zoned| at.minute() == 0 && [1, 3, 5, 7, 9, 11].contains(&at.hour());
    assert_eq!(
        disagreements(&read, &utc("2026-11-03T00:00"), 24 * 60, expected),
        vec![]
    );
}

#[test]
fn both_day_fields_restricted() {
    let read = schedule("0 9 1 * MON");
    let expected = |at: &Zoned| nine(at) && (at.day() == 1 || at.weekday() == Weekday::Monday);
    // Every minute of a week holding a Monday and the first of a month.
    assert_eq!(
        disagreements(&read, &utc("2026-11-28T00:00"), WEEK, expected),
        vec![]
    );
    // And 09:00 every day of two months.
    let mut day = utc("2026-11-01T09:00");
    while day.month() <= 12 && day.year() == 2026 {
        assert_eq!(common::names(&read, &day), expected(&day), "{day}");
        day = day.tomorrow().expect("a day later");
    }
}

#[test]
fn sunday_as_0_and_as_7() {
    for text in ["0 9 * * 0", "0 9 * * 7", "0 9 * * sun", "0 9 * * SUN-SUN"] {
        let read = schedule(text);
        let expected = |at: &Zoned| nine(at) && at.weekday() == Weekday::Sunday;
        assert_eq!(
            disagreements(&read, &utc("2026-11-02T00:00"), 2 * WEEK, expected),
            vec![],
            "{text}"
        );
    }
}

#[test]
fn sunday_ends_a_range_that_starts_after_it() {
    let read = schedule("0 9 * * FRI-SUN");
    let expected = |at: &Zoned| {
        nine(at)
            && matches!(
                at.weekday(),
                Weekday::Friday | Weekday::Saturday | Weekday::Sunday
            )
    };
    assert_eq!(
        disagreements(&read, &utc("2026-11-02T00:00"), WEEK, expected),
        vec![]
    );
}

#[test]
fn a_weekday_with_day_of_month_unrestricted() {
    let read = schedule("0 9 * * MON");
    let last_run = instant("2026-11-02T09:00Z");
    assert!(!is_due(&read, Some(last_run), &utc("2026-11-08T23:59")));
    assert!(is_due(&read, Some(last_run), &utc("2026-11-09T09:00")));
}

#[test]
fn a_day_of_month_with_weekday_unrestricted() {
    let read = schedule("0 9 15 * *");
    let last_run = instant("2026-11-15T09:00Z");
    assert!(!is_due(&read, Some(last_run), &utc("2026-12-14T23:59")));
    assert!(is_due(&read, Some(last_run), &utc("2026-12-15T09:00")));
}

#[test]
fn six_fields() {
    let error = refused("0 0 9 * * *");
    assert_eq!(error, ScheduleError::FieldCount(6));
    assert!(error.to_string().contains("has 6"), "{error}");
}

#[test]
fn four_fields_and_seven() {
    assert_eq!(refused("0 9 * *"), ScheduleError::FieldCount(4));
    assert_eq!(refused("0 0 9 * * * 2026"), ScheduleError::FieldCount(7));
    assert_eq!(refused(""), ScheduleError::FieldCount(0));
}

#[test]
fn a_nickname() {
    for nickname in ["@hourly", "@daily", "@reboot", "@yearly"] {
        let error = refused(nickname);
        assert_eq!(error, ScheduleError::Nickname(nickname.to_string()));
        assert!(error.to_string().contains(nickname), "{error}");
    }
}

#[test]
fn an_extension() {
    let error = refused("0 9 L * *");
    assert_eq!(error, field(Field::DayOfMonth, "L", Reason::Extension('L')));
    assert!(error.to_string().starts_with("day of month `L`"), "{error}");

    for (text, expected) in [
        (
            "0 9 15W * *",
            field(Field::DayOfMonth, "15W", Reason::Extension('W')),
        ),
        (
            "0 9 LW * *",
            field(Field::DayOfMonth, "LW", Reason::Extension('L')),
        ),
        (
            "0 9 * * 5L",
            field(Field::DayOfWeek, "5L", Reason::Extension('L')),
        ),
        (
            "0 9 * * 5#2",
            field(Field::DayOfWeek, "5#2", Reason::Extension('#')),
        ),
        (
            "0 9 ? * MON",
            field(Field::DayOfMonth, "?", Reason::Extension('?')),
        ),
        (
            "0 9 1 * +MON",
            field(Field::DayOfWeek, "+MON", Reason::Extension('+')),
        ),
        (
            "0 9 1,l * *",
            field(Field::DayOfMonth, "l", Reason::Extension('L')),
        ),
    ] {
        assert_eq!(refused(text), expected, "{text}");
    }
}

#[test]
fn an_unknown_name() {
    assert_eq!(
        refused("0 9 * * FUN"),
        field(Field::DayOfWeek, "FUN", Reason::UnknownName)
    );
    assert_eq!(
        refused("0 9 * * MONDAY"),
        field(Field::DayOfWeek, "MONDAY", Reason::UnknownName)
    );
    assert_eq!(
        refused("0 9 * JANUARY *"),
        field(Field::Month, "JANUARY", Reason::UnknownName)
    );
}

#[test]
fn a_name_outside_its_field() {
    for (text, expected) in [
        (
            "0 9 * MON *",
            field(Field::Month, "MON", Reason::NameOutsideField),
        ),
        (
            "0 9 * * JAN",
            field(Field::DayOfWeek, "JAN", Reason::NameOutsideField),
        ),
        (
            "JAN 9 * * *",
            field(Field::Minute, "JAN", Reason::NameOutsideField),
        ),
        (
            "0 MON * * *",
            field(Field::Hour, "MON", Reason::NameOutsideField),
        ),
        (
            "0 9 SUN * *",
            field(Field::DayOfMonth, "SUN", Reason::NameOutsideField),
        ),
    ] {
        assert_eq!(refused(text), expected, "{text}");
    }
}

#[test]
fn a_value_out_of_range() {
    let error = refused("60 * * * *");
    assert_eq!(error, field(Field::Minute, "60", Reason::OutOfRange));
    assert!(error.to_string().starts_with("minute `60`"), "{error}");

    for (text, expected) in [
        ("0 24 * * *", field(Field::Hour, "24", Reason::OutOfRange)),
        (
            "0 9 0 * *",
            field(Field::DayOfMonth, "0", Reason::OutOfRange),
        ),
        (
            "0 9 32 * *",
            field(Field::DayOfMonth, "32", Reason::OutOfRange),
        ),
        ("0 9 * 0 *", field(Field::Month, "0", Reason::OutOfRange)),
        ("0 9 * 13 *", field(Field::Month, "13", Reason::OutOfRange)),
        (
            "0 9 * * 8",
            field(Field::DayOfWeek, "8", Reason::OutOfRange),
        ),
        (
            "0,60 * * * *",
            field(Field::Minute, "60", Reason::OutOfRange),
        ),
        (
            "50-60 * * * *",
            field(Field::Minute, "50-60", Reason::OutOfRange),
        ),
        (
            "99999999999 * * * *",
            field(Field::Minute, "99999999999", Reason::OutOfRange),
        ),
        ("*/0 * * * *", field(Field::Minute, "*/0", Reason::Step)),
        ("*/60 * * * *", field(Field::Minute, "*/60", Reason::Step)),
    ] {
        assert_eq!(refused(text), expected, "{text}");
    }
}

#[test]
fn what_is_not_a_value_a_range_or_a_step() {
    for (text, expected) in [
        (
            "5/10 * * * *",
            field(Field::Minute, "5/10", Reason::Malformed),
        ),
        (
            "/10 * * * *",
            field(Field::Minute, "/10", Reason::Malformed),
        ),
        ("*/ * * * *", field(Field::Minute, "*/", Reason::Malformed)),
        ("*,5 * * * *", field(Field::Minute, "*", Reason::Malformed)),
        ("1,,2 * * * *", field(Field::Minute, "", Reason::Malformed)),
        (
            "1-2-3 * * * *",
            field(Field::Minute, "1-2-3", Reason::Malformed),
        ),
        ("-5 * * * *", field(Field::Minute, "-5", Reason::Malformed)),
        ("0 9 * * %", field(Field::DayOfWeek, "%", Reason::Malformed)),
        (
            "30-10 * * * *",
            field(Field::Minute, "30-10", Reason::Backward),
        ),
    ] {
        assert_eq!(refused(text), expected, "{text}");
    }
}

#[test]
fn a_refused_schedule_is_never_read() {
    // A refusal is an error, never a schedule that names nothing.
    assert!(Schedule::parse("0 9 L * *").is_err());
    // A day that never comes is five-field cron, and names nothing.
    let never = schedule("0 0 31 2 *");
    let now = instant("2026-11-03T00:00Z").to_zoned(TimeZone::UTC);
    assert!(!is_due(&never, Some(instant("2026-01-01T00:00Z")), &now));
}
