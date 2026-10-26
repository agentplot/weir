//! What a host case compares (C.7): the `weir-report/1` a host prints after
//! each step with the step's, and the files at the branch's tip after the
//! last step with the case's `tip/`. Every file and field is compared as
//! written, but the three fields of a capture the host wrote during the
//! case that are the host's and its machine's (22): its `machine`, its
//! `taken_at` and its `pointer`, each held to what the statement says of
//! it (17, 18, 21; core.14).

use std::collections::{BTreeMap, BTreeSet};

use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};
use weir_capture::read_report;

use crate::host::{Expected, HostCase, Offered};

/// What the comparison of a tip is handed of this machine: its short
/// hostname, when the case began and ended, and a way to read every file
/// under a path on it, each with its bytes (core.1).
pub struct Machine<'a> {
    pub hostname: &'a str,
    pub began: OffsetDateTime,
    pub ended: OffsetDateTime,
    pub read: &'a dyn Fn(&str) -> Result<Vec<Vec<u8>>, String>,
}

/// None spelled as the comparisons spell it.
fn or_none<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_string(), |value| format!("`{value}`"))
}

/// How the report `printed`, the last line a host printed after a step,
/// differs from what the step expects; none when it agrees. A refusal is
/// matched by its line and key, its reason holding the step's phrase (15);
/// a run's listing, failures, cursor and completeness are compared besides
/// (8, 9, 23).
pub fn compare_report(expected: &Expected, printed: &str) -> Vec<String> {
    let report = match read_report(printed) {
        Ok(report) => report,
        Err(error) => {
            return vec![format!(
                "the report does not read as weir-report/1: {error}"
            )]
        }
    };
    let mut said = Vec::new();
    let taken: Vec<String> = report.taken.iter().map(ToString::to_string).collect();
    if taken != expected.taken {
        said.push(format!(
            "it took {taken:?}, and the case expects {:?}",
            expected.taken
        ));
    }
    if report.held != expected.held {
        said.push(format!(
            "it held {}, and the case expects {}",
            report.held, expected.held
        ));
    }
    let mut refused = report.refused.clone();
    let named = |line: usize, key: &Option<String>| match key {
        Some(key) => format!("line {line} for `{key}`"),
        None => format!("line {line}"),
    };
    for wanted in &expected.refused {
        match refused
            .iter()
            .position(|refusal| refusal.line == wanted.line && refusal.key == wanted.key)
        {
            None => said.push(format!(
                "it did not refuse {}, and the case expects it refused with a reason holding `{}`",
                named(wanted.line, &wanted.key),
                wanted.reason_holds
            )),
            Some(at) => {
                let refusal = refused.remove(at);
                if !refusal.reason.contains(&wanted.reason_holds) {
                    said.push(format!(
                        "it refused {} because `{}`, and the case expects a reason holding `{}`",
                        named(refusal.line, &refusal.key),
                        refusal.reason,
                        wanted.reason_holds
                    ));
                }
            }
        }
    }
    said.extend(refused.iter().map(|refusal| {
        format!(
            "it refused {} because `{}`, which the case does not expect",
            named(refusal.line, &refusal.key),
            refusal.reason
        )
    }));
    if let Some(run) = &expected.run {
        if report.listed != run.listed {
            said.push(format!(
                "it reports the listing's count as {}, and the case expects {}",
                or_none(report.listed),
                or_none(run.listed)
            ));
        }
        if report.failed != run.failed {
            said.push(format!(
                "it reports the failures {:?}, and the case expects {:?}",
                report.failed, run.failed
            ));
        }
        if report.cursor != run.cursor {
            said.push(format!(
                "it kept the cursor {}, and the case expects {}",
                or_none(report.cursor.as_deref()),
                or_none(run.cursor.as_deref())
            ));
        }
        if report.complete != run.complete {
            said.push(format!(
                "it reports the run {}, and the case expects it {}",
                complete(report.complete),
                complete(run.complete)
            ));
        }
    }
    said
}

fn complete(complete: bool) -> &'static str {
    if complete {
        "complete"
    } else {
        "incomplete"
    }
}

/// How the files at the tip differ from the case's `tip/`; none when they
/// agree. A file the tip holds that `tip/` does not, or lacks, or holds
/// with other text, differs (24); in a capture the host wrote during the
/// case, `machine` is this machine's short hostname, `taken_at` a time
/// within the case's running, and `pointer`, where `tip/` holds one,
/// `<machine>:<path>` with a path here holding every raw file the case's
/// offer named and the whole text of an own excerpt cut, or, for an
/// imported capture, the body's pointer as written (17, 18, 21, 22, 37).
pub fn compare_tip(
    case: &HostCase,
    tip: &BTreeMap<String, Vec<u8>>,
    machine: &Machine<'_>,
) -> Vec<String> {
    let offered = case.offered();
    let paths: BTreeSet<&String> = tip.keys().chain(case.tip.keys()).collect();
    let mut said = Vec::new();
    for path in paths {
        match (tip.get(path), case.tip.get(path)) {
            (Some(_), None) => said.push(format!("the tip holds {path}, which tip/ does not")),
            (None, Some(_)) => said.push(format!("the tip lacks {path}, which tip/ holds")),
            (Some(found), Some(wanted)) if found != wanted => {
                let found = String::from_utf8_lossy(found);
                let wanted = String::from_utf8_lossy(wanted);
                match captured(path).filter(|_| !case.start.contains_key(path.as_str())) {
                    Some(key) => said.extend(compare_capture(
                        path,
                        &found,
                        &wanted,
                        offered.get(key),
                        machine,
                    )),
                    None => said.push(differs(path, &found, &wanted)),
                }
            }
            _ => {}
        }
    }
    said
}

/// The key of the capture whose record lies at `path`, where one does.
fn captured(path: &str) -> Option<&str> {
    path.strip_prefix("inbox/captures/")?
        .strip_suffix("/capture.rec")
}

/// The first line at which `found` differs from `wanted`.
fn differs(path: &str, found: &str, wanted: &str) -> String {
    let found_lines: Vec<&str> = found.split('\n').collect();
    let wanted_lines: Vec<&str> = wanted.split('\n').collect();
    let at = found_lines
        .iter()
        .zip(&wanted_lines)
        .position(|(found, wanted)| found != wanted)
        .unwrap_or(found_lines.len().min(wanted_lines.len()));
    let line = |lines: &[&str]| {
        lines
            .get(at)
            .map_or("nothing".to_string(), |line| format!("`{line}`"))
    };
    format!(
        "{path} differs from tip/'s at line {}: it holds {}, and tip/ holds {}",
        at + 1,
        line(&found_lines),
        line(&wanted_lines)
    )
}

/// How a capture the host wrote differs from tip/'s, line by line, its
/// three fields of the host's own held to their form.
fn compare_capture(
    path: &str,
    found: &str,
    wanted: &str,
    offered: Option<&Offered>,
    machine: &Machine<'_>,
) -> Vec<String> {
    let found_lines: Vec<&str> = found.split('\n').collect();
    let wanted_lines: Vec<&str> = wanted.split('\n').collect();
    if found_lines.len() != wanted_lines.len() {
        return vec![differs(path, found, wanted)];
    }
    let imported = wanted_lines
        .iter()
        .any(|line| line.starts_with("came_in: import:"));
    let field = |line: &str, name: &str| {
        line.strip_prefix(name)?
            .strip_prefix(": ")
            .map(str::to_string)
    };
    let record_machine = found_lines.iter().find_map(|line| field(line, "machine"));
    let mut said = Vec::new();
    for (found_line, wanted_line) in found_lines.iter().zip(&wanted_lines) {
        if let (Some(value), Some(_)) =
            (field(found_line, "machine"), field(wanted_line, "machine"))
        {
            if value != machine.hostname {
                said.push(format!(
                    "{path}: `machine` is `{value}`, and this machine is `{}`",
                    machine.hostname
                ));
            }
        } else if let (Some(value), Some(_)) = (
            field(found_line, "taken_at"),
            field(wanted_line, "taken_at"),
        ) {
            let began = machine.began.replace_nanosecond(0).unwrap_or(machine.began);
            let ended =
                machine.ended.replace_nanosecond(0).unwrap_or(machine.ended) + Duration::SECOND;
            match OffsetDateTime::parse(&value, &Rfc3339) {
                Ok(at) if began <= at && at <= ended => {}
                _ => said.push(format!(
                    "{path}: `taken_at` is `{value}`, not a time within the case's running, from \
                     {} to {}",
                    began.format(&Rfc3339).unwrap_or_default(),
                    ended.format(&Rfc3339).unwrap_or_default()
                )),
            }
        } else if let (Some(value), Some(_), false) = (
            field(found_line, "pointer"),
            field(wanted_line, "pointer"),
            imported,
        ) {
            said.extend(compare_pointer(
                path,
                &value,
                record_machine.as_deref(),
                offered,
                machine,
            ));
        } else if found_line != wanted_line {
            said.push(differs(path, found, wanted));
            break;
        }
    }
    said
}

/// How a pointer the host wrote falls short of what 17, 18 and 21 say of
/// it.
fn compare_pointer(
    path: &str,
    pointer: &str,
    record_machine: Option<&str>,
    offered: Option<&Offered>,
    machine: &Machine<'_>,
) -> Vec<String> {
    let Some((named, place)) = pointer.split_once(':') else {
        return vec![format!(
            "{path}: `pointer` is `{pointer}`, not `<machine>:<path>`"
        )];
    };
    let mut said = Vec::new();
    if Some(named) != record_machine || named != machine.hostname {
        said.push(format!(
            "{path}: `pointer` names the machine `{named}`, and the record's machine is {} and \
             this machine `{}`",
            or_none(record_machine),
            machine.hostname
        ));
    }
    let banked = match (machine.read)(place) {
        Ok(banked) => banked,
        Err(why) => {
            said.push(format!(
                "{path}: `pointer` names `{place}`, which cannot be read here: {why}"
            ));
            return said;
        }
    };
    let Some(offered) = offered else {
        return said;
    };
    for (raw, bytes) in &offered.raw {
        if !banked.contains(bytes) {
            said.push(format!(
                "{path}: `pointer` names `{place}`, which does not hold the raw `{raw}` byte for \
                 byte: the raw file is not banked"
            ));
        }
    }
    if let Some(whole) = &offered.whole_excerpt {
        if !banked.contains(&whole.as_bytes().to_vec()) {
            said.push(format!(
                "{path}: `pointer` names `{place}`, which does not hold the whole text of the own \
                 excerpt it cut"
            ));
        }
    }
    said
}

/// The line `weir conform` prints of a case: its name and `passed`, or
/// `failed` with the clauses it cites and each difference found (65, 87).
pub fn verdict(case: &HostCase, differences: &[String]) -> String {
    if differences.is_empty() {
        return format!("{}: passed", case.name);
    }
    let cites: Vec<String> = case.satisfies.iter().map(ToString::to_string).collect();
    format!(
        "{}: failed, citing {}: {}",
        case.name,
        cites.join(", "),
        differences.join("; ")
    )
}
