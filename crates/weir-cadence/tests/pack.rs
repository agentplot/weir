//! Cadence's cases in the pack (C.7): every case under `pack/cadence/`
//! cites the clauses it exercises (67, core.35) and is answered as it says
//! (59, 60).

use std::fs;
use std::path::{Path, PathBuf};

use jiff::fmt::temporal::DateTimeParser;
use jiff::tz::TimeZoneDatabase;
use jiff::{Timestamp, Zoned};
use serde::Deserialize;
use weir_cadence::{is_due, Schedule};

/// A case as its file writes it: a field the shape does not hold is
/// refused, so a misspelt `last_run` never reads as a thing that has never
/// run.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    satisfies: Vec<u32>,
    schedule: String,
    last_run: Option<String>,
    now: String,
    due: bool,
}

/// A case read whole.
struct Case {
    file: String,
    satisfies: Vec<u32>,
    schedule: Schedule,
    last_run: Option<Timestamp>,
    now: Zoned,
    due: bool,
}

/// Reads the case `text` from `file`, or names the file and the field that
/// fails it.
fn read_case(file: &str, text: &str) -> Result<Case, String> {
    let fails = |field: &str, why: &dyn std::fmt::Display| format!("{file}: {field}: {why}");
    let written: Written =
        toml::from_str(text).map_err(|error| format!("{file}: {}", error.message()))?;
    if written.satisfies.is_empty() {
        return Err(fails("satisfies", &"cites no clause"));
    }
    let schedule = Schedule::parse(&written.schedule).map_err(|error| fails("schedule", &error))?;
    let last_run = written
        .last_run
        .map(|text| text.parse::<Timestamp>())
        .transpose()
        .map_err(|error| fails("last_run", &error))?;
    // The zone is read from the bundled database, as every test's is, and an
    // offset the zone did not have at that instant is refused.
    let now = DateTimeParser::new()
        .parse_zoned_with(&TimeZoneDatabase::bundled(), &written.now)
        .map_err(|error| fails("now", &error))?;
    Ok(Case {
        file: file.to_string(),
        satisfies: written.satisfies,
        schedule,
        last_run,
        now,
        due: written.due,
    })
}

/// The pack's folder of cadence's cases.
fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack/cadence")
}

/// Every `*.toml` under `dir`.
fn case_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            case_files(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            found.push(path);
        }
    }
}

/// Every case of the pack, each named by its path under the folder; fails
/// naming every file that does not read.
fn cases() -> Vec<Case> {
    let folder = folder();
    let mut files = Vec::new();
    case_files(&folder, &mut files);
    files.sort();
    let (read, refused): (Vec<_>, Vec<_>) = files
        .iter()
        .map(|path| {
            let file = path.strip_prefix(&folder).unwrap_or(path).display();
            let text = fs::read_to_string(path).map_err(|error| format!("{file}: {error}"))?;
            read_case(&file.to_string(), &text)
        })
        .partition(Result::is_ok);
    let refused: Vec<String> = refused.into_iter().filter_map(Result::err).collect();
    assert_eq!(refused, Vec::<String>::new(), "cases that do not read");
    read.into_iter().filter_map(Result::ok).collect()
}

/// What a case says, or what cadence says of it.
fn said(due: bool) -> &'static str {
    if due {
        "due"
    } else {
        "not due"
    }
}

#[test]
fn every_case_is_answered_as_it_says() {
    let cases = cases();
    assert!(!cases.is_empty(), "no cases under {}", folder().display());
    let answered_otherwise: Vec<String> = cases
        .iter()
        .filter_map(|case| {
            let answer = is_due(&case.schedule, case.last_run, &case.now);
            (answer != case.due).then(|| {
                format!(
                    "{}: says {}, cadence says {}",
                    case.file,
                    said(case.due),
                    said(answer)
                )
            })
        })
        .collect();
    assert_eq!(answered_otherwise, Vec::<String>::new());
}

#[test]
fn the_two_clauses_are_cited() {
    let cases = cases();
    for clause in [59, 60] {
        assert!(
            cases.iter().any(|case| case.satisfies.contains(&clause)),
            "no case cites {clause}"
        );
    }
}

#[test]
fn a_case_that_never_ran() {
    let never_ran: Vec<Case> = cases()
        .into_iter()
        .filter(|case| case.last_run.is_none())
        .collect();
    assert!(!never_ran.is_empty(), "no case gives no last run");
    for case in never_ran {
        assert!(case.due, "{}: says not due", case.file);
        assert!(
            is_due(&case.schedule, None, &case.now),
            "{}: cadence says not due",
            case.file
        );
    }
}

/// A case of the right shape, with `field` written as `line` in its place,
/// or left out when `line` is empty; with no field, the case as it is.
fn with(field: &str, line: &str) -> String {
    [
        "satisfies = [60]",
        "schedule = \"0 * * * *\"",
        "last_run = \"2026-11-03T11:00:00Z\"",
        "now = \"2026-11-03T09:20:00-05:00[America/New_York]\"",
        "due = true",
    ]
    .iter()
    .map(|written| {
        if written.starts_with(&format!("{field} =")) {
            line
        } else {
            written
        }
    })
    .filter(|written| !written.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

/// The refusal of `text`, read from `wrong.toml`.
fn refusal(text: &str) -> String {
    match read_case("wrong.toml", text) {
        Ok(_) => panic!("read:\n{text}"),
        Err(refusal) => refusal,
    }
}

#[test]
fn a_case_of_the_right_shape_reads() {
    let case = read_case("right.toml", &with("", "")).expect("the case reads");
    assert_eq!(case.file, "right.toml");
    assert_eq!(case.satisfies, vec![60]);
    assert_eq!(case.schedule, Schedule::parse("0 * * * *").unwrap());
    assert_eq!(case.last_run, Some("2026-11-03T11:00:00Z".parse().unwrap()));
    assert_eq!(case.now.time_zone().iana_name(), Some("America/New_York"));
    assert!(case.due);
    let never_ran = read_case("never.toml", &with("last_run", "")).expect("the case reads");
    assert_eq!(never_ran.last_run, None);
}

#[test]
fn a_case_of_the_wrong_shape() {
    let misspelt = refusal(&with("last_run", "last-run = \"2026-11-03T11:00:00Z\""));
    assert!(misspelt.starts_with("wrong.toml: "), "{misspelt}");
    assert!(misspelt.contains("`last-run`"), "{misspelt}");

    let no_zone = refusal(&with("now", "now = \"2026-11-03T09:20:00-05:00\""));
    assert!(no_zone.starts_with("wrong.toml: now: "), "{no_zone}");

    let no_clause = refusal(&with("satisfies", "satisfies = []"));
    assert_eq!(no_clause, "wrong.toml: satisfies: cites no clause");
}

#[test]
fn a_case_that_leaves_out_a_field() {
    for field in ["satisfies", "schedule", "now", "due"] {
        let left_out = refusal(&with(field, ""));
        assert!(left_out.starts_with("wrong.toml: "), "{left_out}");
        assert!(left_out.contains(&format!("`{field}`")), "{left_out}");
    }
}

#[test]
fn a_case_whose_fields_do_not_read() {
    let schedule = refusal(&with("schedule", "schedule = \"@hourly\""));
    assert!(schedule.starts_with("wrong.toml: schedule: "), "{schedule}");

    let last_run = refusal(&with("last_run", "last_run = \"2026-11-03T11:00:00\""));
    assert!(last_run.starts_with("wrong.toml: last_run: "), "{last_run}");

    // New York was at -05:00 on that day, not -04:00.
    let offset = refusal(&with(
        "now",
        "now = \"2026-11-03T09:20:00-04:00[America/New_York]\"",
    ));
    assert!(offset.starts_with("wrong.toml: now: "), "{offset}");
}
