//! The host shape's cases (65; C.7): each holds, in its folder,
//! `case.toml`, naming its clauses, the limit and its steps in order;
//! `start/`, the files the branch holds before the first step, which may
//! be absent; and `tip/`, every file the branch's tip must hold after the
//! last. Each step is one invocation, an intake of a lines file with its
//! raw place where it has one, a run of the case's adapter script, or an
//! import of a file of import lines, and says what the host must report of
//! it (15, 23). A field the format does not name fails the read, naming
//! the case and the field.

use std::collections::BTreeMap;

use serde::Deserialize;
use toml::Value;
use weir_capture::{cut, read_line, Failure, Line};

use crate::clauses::Clause;
use crate::pack::Case;

/// A host case, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCase {
    /// `host/<name>`.
    pub id: String,
    pub name: String,
    pub satisfies: Vec<Clause>,
    /// The limit, in bytes, a host cuts own excerpts at (20).
    pub limit: usize,
    pub steps: Vec<Step>,
    /// The branch's files before the first step, by their paths from its
    /// root.
    pub start: BTreeMap<String, Vec<u8>>,
    /// Every file the tip must hold after the last step.
    pub tip: BTreeMap<String, Vec<u8>>,
    /// The case's own files, by their paths from its folder.
    pub files: BTreeMap<String, Vec<u8>>,
}

/// One step: how the host is invoked, and what it must report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub invocation: Invocation,
    pub expected: Expected,
}

/// How a step invokes the host (C.7), each path a file or folder of the
/// case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    /// The intake: the lines on standard input, the raw place handed with
    /// `--raw-dir` where there is one.
    Intake { lines: String, raw: Option<String> },
    /// A run of the case's adapter, a POSIX shell script, through `--run`:
    /// `lines` and `raw` are what the script prints and writes into the raw
    /// place it is handed.
    Run {
        adapter: String,
        lines: String,
        raw: Option<String>,
    },
    /// An import: the import lines on standard input.
    Import { lines: String },
}

/// What a step's report must hold (15, 23; C.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expected {
    /// The keys taken, in the order of their lines.
    pub taken: Vec<String>,
    pub held: u64,
    pub refused: Vec<ExpectedRefusal>,
    /// For a run, what it must report of the run besides.
    pub run: Option<ExpectedRun>,
}

/// A refusal a step expects: its line, its key where it had one, and a
/// phrase its reason holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedRefusal {
    pub line: usize,
    pub key: Option<String>,
    pub reason_holds: String,
}

/// What a run step must report of the run (8, 9, 23).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedRun {
    /// The listing's count, where a completion was read.
    pub listed: Option<u64>,
    pub failed: Vec<Failure>,
    /// The cursor kept, none for an incomplete run.
    pub cursor: Option<String>,
    pub complete: bool,
}

/// What a case's offers name of a capture, which the pointer the host
/// writes must reach: every raw file's bytes, and the whole text of an own
/// excerpt cut at the limit (17, 18, 21).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offered {
    pub raw: Vec<(String, Vec<u8>)>,
    pub whole_excerpt: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    satisfies: Vec<Value>,
    limit: usize,
    step: Vec<WrittenStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStep {
    intake: Option<String>,
    run: Option<String>,
    import: Option<String>,
    lines: Option<String>,
    raw: Option<String>,
    taken: Vec<String>,
    held: u64,
    #[serde(default)]
    refused: Vec<WrittenRefusal>,
    listed: Option<u64>,
    failed: Option<Vec<WrittenFailure>>,
    cursor: Option<String>,
    complete: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRefusal {
    line: usize,
    key: Option<String>,
    reason_holds: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenFailure {
    what: String,
    why: String,
}

/// Reads the host case `case`, or names the case and why it does not
/// read: a field it fails on, a step of no invocation or of two, a field
/// a step of its invocation does not hold, or a file it names that the
/// case does not hold.
pub fn read_host_case(case: &Case) -> Result<HostCase, String> {
    let id = case.id();
    let refused = |why: String| format!("{id}: {why}");
    let written: Written = toml::from_str(case.text("case.toml")?)
        .map_err(|error| refused(error.message().to_string()))?;
    let satisfies = written
        .satisfies
        .iter()
        .map(Clause::cited)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|why| refused(format!("`satisfies`: {why}")))?;
    if written.step.is_empty() {
        return Err(refused("holds no step".into()));
    }
    let names_file = |n: usize, field: &str, path: &str| -> Result<(), String> {
        if case.file(path).is_some() {
            Ok(())
        } else {
            Err(refused(format!(
                "step {n}: `{field}` names `{path}`, which is no file of the case"
            )))
        }
    };
    let names_folder = |n: usize, path: &str| -> Result<(), String> {
        if case.under(path).is_empty() {
            Err(refused(format!(
                "step {n}: `raw` names `{path}`, which is no folder of the case"
            )))
        } else {
            Ok(())
        }
    };
    let mut steps = Vec::with_capacity(written.step.len());
    for (n, step) in (1..).zip(written.step) {
        let invocation = match (step.intake, step.run, step.import) {
            (Some(lines), None, None) => {
                if step.lines.is_some() {
                    return Err(refused(format!(
                        "step {n}: `lines` is a run's, and an intake's lines are its `intake`"
                    )));
                }
                names_file(n, "intake", &lines)?;
                Invocation::Intake {
                    lines,
                    raw: step.raw,
                }
            }
            (None, Some(adapter), None) => {
                let lines = step.lines.ok_or_else(|| {
                    refused(format!(
                        "step {n}: `lines` is missing, and a run names the lines its script prints"
                    ))
                })?;
                names_file(n, "run", &adapter)?;
                names_file(n, "lines", &lines)?;
                Invocation::Run {
                    adapter,
                    lines,
                    raw: step.raw,
                }
            }
            (None, None, Some(lines)) => {
                for (field, given) in [("lines", step.lines.is_some()), ("raw", step.raw.is_some())]
                {
                    if given {
                        return Err(refused(format!(
                            "step {n}: `{field}` is no import's, whose lines are its `import`"
                        )));
                    }
                }
                names_file(n, "import", &lines)?;
                Invocation::Import { lines }
            }
            _ => {
                return Err(refused(format!(
                    "step {n} names no invocation or more than one: exactly one of `intake`, \
                     `run` and `import`"
                )))
            }
        };
        if let Invocation::Intake { raw: Some(raw), .. } | Invocation::Run { raw: Some(raw), .. } =
            &invocation
        {
            names_folder(n, raw)?;
        }
        let run = if let Invocation::Run { .. } = invocation {
            Some(ExpectedRun {
                listed: step.listed,
                failed: step
                    .failed
                    .unwrap_or_default()
                    .into_iter()
                    .map(|failure| Failure {
                        what: failure.what,
                        why: failure.why,
                    })
                    .collect(),
                cursor: step.cursor,
                complete: step.complete.ok_or_else(|| {
                    refused(format!(
                        "step {n}: `complete` is missing, and a run says whether it was complete"
                    ))
                })?,
            })
        } else {
            for (field, given) in [
                ("listed", step.listed.is_some()),
                ("failed", step.failed.is_some()),
                ("cursor", step.cursor.is_some()),
                ("complete", step.complete.is_some()),
            ] {
                if given {
                    return Err(refused(format!("step {n}: `{field}` is a run's alone")));
                }
            }
            None
        };
        steps.push(Step {
            invocation,
            expected: Expected {
                taken: step.taken,
                held: step.held,
                refused: step
                    .refused
                    .into_iter()
                    .map(|refusal| ExpectedRefusal {
                        line: refusal.line,
                        key: refusal.key,
                        reason_holds: refusal.reason_holds,
                    })
                    .collect(),
                run,
            },
        });
    }
    Ok(HostCase {
        id,
        name: case.name.clone(),
        satisfies,
        limit: written.limit,
        steps,
        start: case.under("start"),
        tip: case.under("tip"),
        files: case.files.clone(),
    })
}

impl HostCase {
    /// What the case's offers name of each capture, by its key: those of
    /// the first offer of each key among the steps' lines, each raw file
    /// read from the step's raw place, and an own excerpt's whole text
    /// where the limit cuts it.
    pub fn offered(&self) -> BTreeMap<String, Offered> {
        let mut offered = BTreeMap::new();
        for step in &self.steps {
            let (lines, raw) = match &step.invocation {
                Invocation::Intake { lines, raw } | Invocation::Run { lines, raw, .. } => {
                    (lines, raw.as_deref())
                }
                Invocation::Import { .. } => continue,
            };
            let text =
                String::from_utf8_lossy(self.files.get(lines).map_or(&[][..], Vec::as_slice))
                    .into_owned();
            for line in text.lines() {
                let Ok(Line::Offer(offer)) = read_line(line) else {
                    continue;
                };
                let key = offer.key.to_string();
                if offered.contains_key(&key) {
                    continue;
                }
                let raw_files = offer
                    .raw
                    .iter()
                    .filter_map(|path| {
                        let place = raw?;
                        let bytes = self.files.get(&format!("{place}/{path}"))?;
                        Some((path.clone(), bytes.clone()))
                    })
                    .collect();
                let whole_excerpt = offer
                    .excerpt
                    .filter(|excerpt| cut(&excerpt.text, self.limit).left_out.is_some())
                    .map(|excerpt| excerpt.text);
                offered.insert(
                    key,
                    Offered {
                        raw: raw_files,
                        whole_excerpt,
                    },
                );
            }
        }
        offered
    }
}
