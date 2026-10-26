//! The adapter shape's cases (65; C.7), of two kinds. A case of lines fixes
//! a run's lines in (`lines.jsonl`), its raw place where it has one
//! (`raw/`), whether it exited reporting success, whether it is checked as
//! a meeting adapter with the fixture's facts, and the verdict the checks
//! must give out. A case of a built-in adapter names the adapter, its
//! input, a folder or a file of the case, the modification times its files
//! are given, and the lines it must print (`lines.jsonl`), which must also
//! pass the checks (61, 62, 64). Each reads whole, and a field the format
//! does not name fails the read, naming the case and the field.

use std::collections::BTreeMap;

use serde::Deserialize;
use toml::{Table, Value};

use crate::adapter::{check_lines, check_meeting_lines, Breach, Meetings, RawPlace};
use crate::pack::Case;

/// The file of a case that holds its lines: in, or expected out.
pub const LINES: &str = "lines.jsonl";

/// The folder of a case that is its run's raw place.
pub const RAW: &str = "raw";

/// An adapter case, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterCase {
    Lines(LinesCase),
    BuiltIn(BuiltInCase),
}

/// A case of lines in and the verdict out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinesCase {
    pub id: String,
    pub lines: Vec<String>,
    /// The raw place's files after the run, by their paths from it.
    pub raw: BTreeMap<String, Vec<u8>>,
    pub exited_ok: bool,
    /// The fixture's facts, where the case is checked as a meeting
    /// adapter's run.
    pub meeting: Option<Meetings>,
    /// The breaches the checks must name: none for lines that pass.
    pub breaches: Vec<ExpectedBreach>,
}

/// A breach a case expects: its line, none for the run as a whole, the
/// clause, and a phrase its reason holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedBreach {
    pub line: Option<usize>,
    pub clause: u32,
    pub reason_holds: String,
}

/// A built-in adapter a case names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltIn {
    /// The folder drop, given a folder (61).
    Folder,
    /// The meeting file, given a file (62).
    Meeting,
}

/// A case of a built-in adapter's input and the lines it must print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltInCase {
    pub id: String,
    pub adapter: BuiltIn,
    /// The input's path in the case's folder.
    pub input: String,
    /// The input's files by their paths from it, a file input's as its own
    /// name, each with its bytes.
    pub files: BTreeMap<String, Vec<u8>>,
    /// The modification time each file of the input is given, by its path
    /// from the input, in RFC 3339.
    pub modified: BTreeMap<String, String>,
    pub expected: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenLines {
    #[allow(dead_code)]
    satisfies: Vec<Value>,
    exited_ok: bool,
    passes: bool,
    meeting: Option<WrittenMeeting>,
    #[serde(default)]
    breach: Vec<WrittenBreach>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenMeeting {
    starts: BTreeMap<String, String>,
    short: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenBreach {
    line: Option<usize>,
    clause: u32,
    reason_holds: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenBuiltIn {
    #[allow(dead_code)]
    satisfies: Vec<Value>,
    adapter: String,
    input: String,
    #[serde(default)]
    modified: BTreeMap<String, String>,
}

/// The lines of a case's `lines.jsonl`, one a line.
fn lines_of(case: &Case) -> Result<Vec<String>, String> {
    Ok(case.text(LINES)?.lines().map(str::to_string).collect())
}

/// Reads the adapter case `case`, or names the case and why it does not
/// read: the field it fails on, or the file it lacks.
pub fn read_adapter_case(case: &Case) -> Result<AdapterCase, String> {
    let id = case.id();
    let text = case.text("case.toml")?;
    let table: Table = text
        .parse()
        .map_err(|error: toml::de::Error| format!("{id}: {}", error.message()))?;
    if table.contains_key("adapter") {
        let written: WrittenBuiltIn =
            toml::from_str(text).map_err(|error| format!("{id}: {}", error.message()))?;
        let adapter = match written.adapter.as_str() {
            "folder" => BuiltIn::Folder,
            "meeting" => BuiltIn::Meeting,
            other => {
                return Err(format!(
                    "{id}: `adapter` is `{other}`, not `folder` or `meeting`"
                ))
            }
        };
        let files = match case.file(&written.input) {
            Some(bytes) => {
                let name = written.input.rsplit('/').next().unwrap_or(&written.input);
                BTreeMap::from([(name.to_string(), bytes.to_vec())])
            }
            None => case.under(&written.input),
        };
        if files.is_empty() {
            return Err(format!(
                "{id}: `input` is `{}`, which names no file or folder of the case",
                written.input
            ));
        }
        if let Some(stray) = written
            .modified
            .keys()
            .find(|path| !files.contains_key(*path))
        {
            return Err(format!(
                "{id}: `modified` names `{stray}`, which is no file of the input"
            ));
        }
        return Ok(AdapterCase::BuiltIn(BuiltInCase {
            id,
            adapter,
            input: written.input,
            files,
            modified: written.modified,
            expected: lines_of(case)?,
        }));
    }
    let written: WrittenLines =
        toml::from_str(text).map_err(|error| format!("{id}: {}", error.message()))?;
    if written.passes != written.breach.is_empty() {
        return Err(format!(
            "{id}: `passes` is {}, and the case names {} breaches: lines that pass name none, \
             and lines that fail name each",
            written.passes,
            written.breach.len()
        ));
    }
    Ok(AdapterCase::Lines(LinesCase {
        id,
        lines: lines_of(case)?,
        raw: case.under(RAW),
        exited_ok: written.exited_ok,
        meeting: written.meeting.map(|meeting| Meetings {
            starts: meeting.starts,
            short: meeting.short,
        }),
        breaches: written
            .breach
            .into_iter()
            .map(|breach| ExpectedBreach {
                line: breach.line,
                clause: breach.clause,
                reason_holds: breach.reason_holds,
            })
            .collect(),
    }))
}

impl LinesCase {
    /// Every breach the checks name of its lines: the meeting checks for a
    /// meeting adapter's run, the adapter checks for any other.
    pub fn breaches(&self) -> Vec<Breach> {
        let raw = RawPlace::Files(&self.raw);
        match &self.meeting {
            Some(meetings) => check_meeting_lines(&self.lines, raw, self.exited_ok, meetings),
            None => check_lines(&self.lines, raw, self.exited_ok),
        }
    }

    /// How the checks' verdict differs from the case's, one difference a
    /// line naming the case; none when they agree. Each expected breach is
    /// matched by its line and clause to one the checks name whose reason
    /// holds its phrase.
    pub fn differences(&self) -> Vec<String> {
        let mut named = self.breaches();
        let mut differences = Vec::new();
        for expected in &self.breaches {
            let found = named.iter().position(|breach| {
                breach.line == expected.line
                    && breach.clause == expected.clause
                    && breach.why.contains(&expected.reason_holds)
            });
            match found {
                Some(at) => {
                    named.remove(at);
                }
                None => {
                    differences.push(format!(
                    "{}: expects {} to break {} with a reason holding `{}`, and the checks name \
                     no such breach",
                    self.id,
                    expected.line.map_or("the run".to_string(), |line| format!("line {line}")),
                    expected.clause,
                    expected.reason_holds
                ))
                }
            }
        }
        differences.extend(named.into_iter().map(|breach| {
            format!(
                "{}: the checks name {breach}, which the case does not expect",
                self.id
            )
        }));
        differences
    }
}
