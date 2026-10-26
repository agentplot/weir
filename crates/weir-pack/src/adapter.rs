//! The adapter checks: what an adapter's lines must be, held line by line
//! to the contract (1 to 12; C.1; core.5, core.6), and the meeting checks
//! a meeting adapter passes besides (66). Both are functions an adapter's
//! own tests call over its output, reading nothing but the lines, the raw
//! place and the facts handed in (68).

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{closing_line, read_line, LeftOut, Line, Malformed, Offer};

/// The source every capture a meeting adapter offers names (66).
pub const MEETING: &str = "meeting";

/// One breach of the contract: the line, counting from 1, where it has
/// one, the clause it breaks, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breach {
    pub line: Option<usize>,
    pub clause: u32,
    pub why: String,
}

impl fmt::Display for Breach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line} breaks {}: {}", self.clause, self.why),
            None => write!(f, "the run breaks {}: {}", self.clause, self.why),
        }
    }
}

/// The raw place a run was handed, as it stands after the run.
#[derive(Clone, Copy, Debug)]
pub enum RawPlace<'a> {
    /// A folder on this machine.
    Folder(&'a Path),
    /// The files it holds, by their paths from it.
    Files(&'a BTreeMap<String, Vec<u8>>),
}

impl RawPlace<'_> {
    /// Whether `path`, relative to the raw place, names a file it holds,
    /// or why not.
    fn holds(&self, path: &str) -> Result<(), String> {
        match self {
            RawPlace::Folder(place) => {
                let place = place
                    .canonicalize()
                    .map_err(|error| format!("the raw place cannot be read: {error}"))?;
                let file = place
                    .join(path)
                    .canonicalize()
                    .map_err(|_| "the raw place does not hold it after the run".to_string())?;
                if !file.starts_with(&place) {
                    return Err("it resolves outside the raw place".to_string());
                }
                if !file.is_file() {
                    return Err("the raw place holds no file there after the run".to_string());
                }
                Ok(())
            }
            RawPlace::Files(files) => {
                let named: Vec<&str> = path
                    .split('/')
                    .filter(|part| !part.is_empty() && *part != ".")
                    .collect();
                if files.contains_key(&named.join("/")) {
                    Ok(())
                } else {
                    Err("the raw place does not hold it after the run".to_string())
                }
            }
        }
    }
}

/// The clause a malformed line breaks, by the field it fails on (C.1).
fn clause_of(malformed: &Malformed) -> u32 {
    let missing = malformed.reason == "is missing";
    match malformed.field.as_deref() {
        None | Some("capture" | "cursor" | "done") => 1,
        Some("key") if missing => 2,
        Some("key") => 4,
        Some("event_at") if missing => 2,
        Some("event_at") => 5,
        Some("source" | "title" | "url") => 2,
        Some(field) if field == "raw" || field.starts_with("raw[") => 6,
        Some(field) if field.starts_with("excerpt") => 7,
        Some(field) if field.starts_with("done.") => 9,
        Some(_) => 1,
    }
}

/// Whether `text` ends with the line a host adds when it cuts an excerpt
/// (20), which only a host writes (12).
fn ends_with_closing_line(text: &str) -> bool {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let last = text.rsplit('\n').next().unwrap_or(text);
    let numbers = last.strip_prefix("[cut: ").and_then(|rest| {
        let (lines, rest) = rest.split_once(" lines and ")?;
        let (bytes, _) = rest.split_once(" bytes")?;
        Some(LeftOut {
            lines: lines.parse().ok()?,
            bytes: bytes.parse().ok()?,
        })
    });
    numbers.is_some_and(|left_out| closing_line(left_out) == last)
}

/// The same instant, read from two RFC 3339 times; text that does not
/// read is compared as written.
fn same_time(one: &str, other: &str) -> bool {
    match (
        OffsetDateTime::parse(one, &Rfc3339),
        OffsetDateTime::parse(other, &Rfc3339),
    ) {
        (Ok(one), Ok(other)) => one == other,
        _ => one == other,
    }
}

/// A run's lines as the checks read them: each line that read, by its
/// number, and every breach of the adapter checks.
struct Read {
    lines: Vec<(usize, Line)>,
    breaches: Vec<Breach>,
}

fn read_run<L: AsRef<str>>(lines: &[L], raw: RawPlace<'_>, exited_ok: bool) -> Read {
    let mut read = Vec::new();
    let mut breaches = Vec::new();
    let mut breach =
        |line: Option<usize>, clause: u32, why: String| breaches.push(Breach { line, clause, why });
    let mut offered: BTreeMap<String, (usize, String, String)> = BTreeMap::new();
    let mut completion: Option<usize> = None;
    for (number, text) in (1..).zip(lines) {
        let line = match read_line(text.as_ref()) {
            Err(malformed) => {
                breach(Some(number), clause_of(&malformed), malformed.to_string());
                continue;
            }
            Ok(line) => line,
        };
        if let Some(done_at) = completion {
            let why = match line {
                Line::Done(_) => format!("is a second completion, after line {done_at}'s"),
                _ => format!("comes after line {done_at}'s completion, which ends the run"),
            };
            breach(Some(number), 9, why);
        }
        match &line {
            Line::Offer(offer) => check_offer(number, offer, raw, &mut offered, &mut breach),
            Line::Cursor(_) => {}
            Line::Done(done) => {
                completion.get_or_insert(number);
                for (n, failure) in done.failed.iter().enumerate() {
                    for (field, value) in [("what", &failure.what), ("why", &failure.why)] {
                        if value.is_empty() {
                            breach(
                                Some(number),
                                9,
                                format!("`done.failed[{n}].{field}` is empty"),
                            );
                        }
                    }
                }
            }
        }
        read.push((number, line));
    }
    if exited_ok && completion.is_none() {
        breach(
            None,
            9,
            "it exited reporting success with no completion".to_string(),
        );
    }
    Read {
        lines: read,
        breaches,
    }
}

fn check_offer(
    number: usize,
    offer: &Offer,
    raw: RawPlace<'_>,
    offered: &mut BTreeMap<String, (usize, String, String)>,
    breach: &mut impl FnMut(Option<usize>, u32, String),
) {
    let key = offer.key.to_string();
    match offered.get(&key) {
        Some((first, source, event_at)) => {
            if *source != offer.source || !same_time(event_at, &offer.event_at) {
                breach(
                    Some(number),
                    3,
                    format!(
                        "offers `{key}`, as line {first} does, with another source or event \
                         time: two events share a key"
                    ),
                );
            }
        }
        None => {
            offered.insert(
                key.clone(),
                (number, offer.source.clone(), offer.event_at.clone()),
            );
        }
    }
    for path in &offer.raw {
        if let Err(why) = raw.holds(path) {
            breach(Some(number), 6, format!("names raw `{path}`, and {why}"));
        }
    }
    if let Some(excerpt) = &offer.excerpt {
        if ends_with_closing_line(&excerpt.text) {
            breach(
                Some(number),
                12,
                "its own excerpt ends with the line a host adds when it cuts one, and an \
                 adapter never cuts an excerpt"
                    .to_string(),
            );
        }
    }
}

/// The adapter checks: every breach of the contract `lines` make, the
/// lines of one run, given the raw place it was handed as it stands after
/// the run and whether it exited reporting success; none when they keep
/// the contract. A breach stops no other line being checked (10).
pub fn check_lines<L: AsRef<str>>(lines: &[L], raw: RawPlace<'_>, exited_ok: bool) -> Vec<Breach> {
    read_run(lines, raw, exited_ok).breaches
}

/// What a meeting adapter's own fixture says, which its lines alone
/// cannot show (66): each meeting's start, by the key it is offered under,
/// and how many meetings it held too short to hold anything.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meetings {
    pub starts: BTreeMap<String, String>,
    pub short: u64,
}

/// The meeting checks: every breach of the adapter checks `lines` make,
/// and every breach of 66 besides, in the order of their lines: an offer
/// keyed other than `meeting/<service>/<the meeting's own identity>`, the
/// identity one of the fixture's meetings; whose source is not `meeting`;
/// whose event time is not its meeting's start; that names fewer than two
/// raw files, the transcript as served and a plain copy; or that carries an
/// own excerpt; and a completion whose count is not the offers, the
/// failures and the meetings left out together. Lines that pass them pass
/// the adapter checks.
pub fn check_meeting_lines<L: AsRef<str>>(
    lines: &[L],
    raw: RawPlace<'_>,
    exited_ok: bool,
    meetings: &Meetings,
) -> Vec<Breach> {
    let Read {
        lines: read,
        mut breaches,
    } = read_run(lines, raw, exited_ok);
    let offers = read
        .iter()
        .filter(|(_, line)| matches!(line, Line::Offer(_)))
        .count() as u64;
    let mut breach = |line: usize, why: String| {
        breaches.push(Breach {
            line: Some(line),
            clause: 66,
            why,
        })
    };
    for (number, line) in &read {
        match line {
            Line::Offer(offer) => {
                let key = offer.key.as_str();
                let segments = offer.key.segments();
                match meetings.starts.get(key) {
                    _ if segments.len() != 3 || segments[0] != MEETING => breach(
                        *number,
                        format!(
                            "is keyed `{key}`, not `meeting/<service>/<the meeting's own identity>`"
                        ),
                    ),
                    None => breach(
                        *number,
                        format!("is keyed `{key}`, which names no meeting of the fixture"),
                    ),
                    Some(start) if !same_time(start, &offer.event_at) => breach(
                        *number,
                        format!(
                            "has the event time `{}`, and its meeting started at `{start}`",
                            offer.event_at
                        ),
                    ),
                    Some(_) => {}
                }
                if offer.source != MEETING {
                    breach(
                        *number,
                        format!("names the source `{}`, not `{MEETING}`", offer.source),
                    );
                }
                if offer.raw.len() < 2 {
                    breach(
                        *number,
                        format!(
                            "names {} raw files, and a meeting's are two: the transcript as \
                             served and a plain copy",
                            offer.raw.len()
                        ),
                    );
                }
                if offer.excerpt.is_some() {
                    breach(
                        *number,
                        "carries an own excerpt, and a meeting's capture carries none".to_string(),
                    );
                }
            }
            Line::Done(done) => {
                let failed = done.failed.len() as u64;
                let counted = offers + failed + meetings.short;
                if done.listed != counted {
                    breach(
                        *number,
                        format!(
                            "lists {}, and the run offered {offers}, failed {failed} and left \
                             out {} meetings too short to hold anything: {counted} together",
                            done.listed, meetings.short
                        ),
                    );
                }
            }
            Line::Cursor(_) => {}
        }
    }
    // In the order of their lines, a breach of the whole run last.
    breaches.sort_by_key(|breach| breach.line.unwrap_or(usize::MAX));
    breaches
}
