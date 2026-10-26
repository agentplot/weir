//! The pack's inbox cases, `pack/inbox/<case>/`, each read from its own
//! files and run against the crate, held to the scenarios of inbox-cases:
//! a starting inbox read with `InboxAt::read`, one operation formed on it,
//! and the tip its write leaves compared with `tip/` file for file and byte
//! for byte, or its refusal with the one the case expects (25 to 43, 63,
//! 65, 67, 68; C.2, C.7; core.24, core.36).

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{read_import_line, Key, RawFile};
use weir_signals::{
    ImportedCapture, InboxAt, NewMove, NotTaken, Outcome, ReadSignal, Refused, SignalId,
};

use common::recfix;

/// A case as its `case.toml` writes it: a field the format does not name
/// fails the read, naming it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    satisfies: Vec<toml::Value>,
    operation: Operation,
    #[serde(default)]
    refused: Vec<ExpectedRefusal>,
}

/// The one operation a case forms on its starting inbox (C.2; C.7).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Operation {
    Reading(Reading),
    Move(MoveArgs),
    Replacement(MoveArgs),
    ProposeBatch(ProposeBatch),
    DecideBatch(DecideBatch),
    ProposeItem(ProposeItem),
    WriteBrief(WriteBrief),
    Send(Send),
    SettleSend(SettleSend),
    Import(Import),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reading {
    key: String,
    /// The capture's raw files, each a file of the case.
    #[serde(default)]
    raw: Vec<String>,
    #[serde(default)]
    signal: Vec<SignalArgs>,
}

/// A signal as a reader hands it; a field left out is handed empty, for
/// the reading to refuse.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct SignalArgs {
    kind: String,
    asserted_by: String,
    assertion: String,
    subjects: Vec<String>,
    argues_with: Vec<String>,
    excerpt: String,
    position: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveArgs {
    signal: String,
    word: String,
    #[serde(default)]
    target: String,
    note: Option<String>,
    #[serde(default)]
    by: String,
    #[serde(default)]
    at: String,
    replaces: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposeBatch {
    batch: String,
    by: String,
    at: String,
    #[serde(rename = "move")]
    moves: Vec<MoveArgs>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecideBatch {
    batch: String,
    stands: bool,
    by: String,
    at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposeItem {
    item: String,
    conclusion: String,
    by: String,
    at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteBrief {
    item: String,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Send {
    items: Vec<String>,
    target: String,
    sent_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettleSend {
    item: String,
    target: String,
    sent_at: String,
    outcome: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    /// The file of the case holding its import lines.
    lines: String,
    machine: String,
    imported_at: String,
}

/// One thing a refusal must not take: what it names, its field where it
/// has one, and a phrase its reason holds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedRefusal {
    what: String,
    field: Option<String>,
    reason_holds: String,
}

/// A case read whole: its name, its case file, and its files by their
/// paths from its folder.
struct Case {
    name: String,
    written: Written,
    files: BTreeMap<String, Vec<u8>>,
}

/// The pack's folder of inbox cases.
fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack/inbox")
}

/// Every file under `dir`, by `prefix` and its path from `dir`.
fn files_under(dir: &Path, prefix: &str, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let entry = entry.unwrap();
        let path = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if entry.path().is_dir() {
            files_under(&entry.path(), &format!("{path}/"), files);
        } else {
            files.insert(path, fs::read(entry.path()).unwrap());
        }
    }
}

/// Reads the case `name` of `files`, or names the case and why not.
fn read_case(name: &str, files: BTreeMap<String, Vec<u8>>) -> Result<Case, String> {
    let text = files
        .get("case.toml")
        .ok_or_else(|| format!("{name}: holds no case.toml"))?;
    let text = std::str::from_utf8(text).map_err(|_| format!("{name}: case.toml is not UTF-8"))?;
    let written: Written =
        toml::from_str(text).map_err(|error| format!("{name}: {}", error.message()))?;
    if written.satisfies.is_empty() {
        return Err(format!("{name}: `satisfies` cites no clause"));
    }
    let tip = files.keys().any(|path| path.starts_with("tip/"));
    if tip == !written.refused.is_empty() {
        return Err(format!(
            "{name}: a case expects a tip/ or a refusal, exactly one of them"
        ));
    }
    Ok(Case {
        name: name.to_string(),
        written,
        files,
    })
}

/// Every case of the pack, in the order of their names.
fn cases() -> Vec<Case> {
    let mut names: Vec<String> = fs::read_dir(folder())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| {
            let mut files = BTreeMap::new();
            files_under(&folder().join(name), "", &mut files);
            read_case(name, files).unwrap_or_else(|why| panic!("{why}"))
        })
        .collect()
}

impl Case {
    /// The files under the case's folder `under`, by their paths from it,
    /// each as text.
    fn texts(&self, under: &str) -> BTreeMap<String, String> {
        let prefix = format!("{under}/");
        self.files
            .iter()
            .filter_map(|(path, bytes)| {
                let path = path.strip_prefix(&prefix)?;
                let text = String::from_utf8(bytes.clone())
                    .unwrap_or_else(|_| panic!("{}: {under}/{path} is not UTF-8", self.name));
                Some((path.to_string(), text))
            })
            .collect()
    }

    fn file(&self, path: &str) -> &[u8] {
        self.files
            .get(path)
            .unwrap_or_else(|| panic!("{}: holds no `{path}`", self.name))
    }
}

fn new_move(args: &MoveArgs) -> Result<NewMove, Refused> {
    let signal = SignalId::parse(&args.signal).map_err(|why| {
        Refused(vec![NotTaken {
            what: args.signal.clone(),
            field: Some("signal".into()),
            why,
        }])
    })?;
    Ok(NewMove {
        signal,
        word: args.word.clone(),
        target: args.target.clone(),
        note: args.note.clone(),
        by: args.by.clone(),
        at: args.at.clone(),
        replaces: args.replaces,
    })
}

/// The tip the case's operation leaves, by path from the tip's root, or
/// what refused it: a starting inbox that does not read, or the operation.
fn run(case: &Case) -> Result<BTreeMap<String, String>, Refused> {
    let start = case.texts("start");
    let inbox = InboxAt::read(&start).map_err(|error| Refused(vec![NotTaken::from(error)]))?;
    let write = match &case.written.operation {
        Operation::Reading(reading) => {
            let key = Key::parse(&reading.key).unwrap_or_else(|why| panic!("{}: {why}", case.name));
            let raw: Vec<RawFile> = reading
                .raw
                .iter()
                .map(|path| RawFile {
                    path: path.clone(),
                    bytes: case.file(path).to_vec(),
                })
                .collect();
            let signals: Vec<ReadSignal> = reading
                .signal
                .iter()
                .map(|signal| ReadSignal {
                    kind: signal.kind.clone(),
                    asserted_by: signal.asserted_by.clone(),
                    assertion: signal.assertion.clone(),
                    subjects: signal.subjects.clone(),
                    argues_with: signal.argues_with.clone(),
                    excerpt: signal.excerpt.clone(),
                    position: signal.position.clone(),
                })
                .collect();
            Some(inbox.read_capture(&key, &signals, &raw)?)
        }
        Operation::Move(args) => Some(inbox.make_move(&new_move(args)?)?),
        Operation::Replacement(args) => {
            assert!(
                args.replaces.is_some(),
                "{}: a replacement names the move it replaces",
                case.name
            );
            Some(inbox.make_move(&new_move(args)?)?)
        }
        Operation::ProposeBatch(batch) => {
            let moves = batch
                .moves
                .iter()
                .map(new_move)
                .collect::<Result<Vec<_>, _>>()?;
            Some(inbox.propose_batch(&batch.batch, &batch.by, &batch.at, &moves)?)
        }
        Operation::DecideBatch(decision) => Some(inbox.decide_batch(
            &decision.batch,
            decision.stands,
            &decision.by,
            &decision.at,
        )?),
        Operation::ProposeItem(item) => {
            Some(inbox.propose_item(&item.item, &item.conclusion, &item.by, &item.at)?)
        }
        Operation::WriteBrief(brief) => Some(inbox.write_brief(&brief.item, &brief.text)?),
        Operation::Send(send) => {
            let items: Vec<&str> = send.items.iter().map(String::as_str).collect();
            Some(inbox.send(&items, &send.target, &send.sent_at)?)
        }
        Operation::SettleSend(settle) => {
            let outcome = Outcome::parse(&settle.outcome)
                .unwrap_or_else(|| panic!("{}: `{}` is no outcome", case.name, settle.outcome));
            Some(inbox.settle_send(&settle.item, &settle.target, &settle.sent_at, &outcome)?)
        }
        Operation::Import(import) => {
            let text = std::str::from_utf8(case.file(&import.lines)).unwrap();
            let captures: Vec<ImportedCapture> = text
                .lines()
                .map(|line| {
                    read_import_line(line)
                        .unwrap_or_else(|why| panic!("{}: {why}: {line}", case.name))
                        .into()
                })
                .collect();
            let at = OffsetDateTime::parse(&import.imported_at, &Rfc3339)
                .unwrap_or_else(|why| panic!("{}: `imported_at`: {why}", case.name));
            inbox.import(&captures, &import.machine, at)?.write
        }
    };
    let mut tip = start;
    if let Some(write) = write {
        tip.extend(write.files);
    }
    Ok(tip)
}

/// How what the operation did differs from what the case expects, each
/// difference naming the case; none when they agree.
fn differences(case: &Case) -> Vec<String> {
    let name = &case.name;
    match (run(case), case.written.refused.as_slice()) {
        (Ok(tip), []) => {
            let expected = case.texts("tip");
            let paths: BTreeSet<&String> = tip.keys().chain(expected.keys()).collect();
            paths
                .into_iter()
                .filter_map(|path| match (tip.get(path), expected.get(path)) {
                    (Some(_), None) => {
                        Some(format!("{name}: the tip holds {path}, which tip/ does not"))
                    }
                    (None, Some(_)) => {
                        Some(format!("{name}: the tip lacks {path}, which tip/ holds"))
                    }
                    (Some(text), Some(wanted)) if text != wanted => Some(format!(
                        "{name}: {path} differs:\n{text}\nand tip/ holds\n{wanted}"
                    )),
                    _ => None,
                })
                .collect()
        }
        (Ok(tip), _) => vec![format!(
            "{name}: expects a refusal, and the write was taken, leaving {:?}",
            tip.keys().collect::<Vec<_>>()
        )],
        (Err(refused), []) => vec![format!(
            "{name}: expects a tip, and it was refused: {refused}"
        )],
        (Err(Refused(not_taken)), expected) => {
            let matches = |not: &NotTaken, wanted: &ExpectedRefusal| {
                not.what == wanted.what
                    && not.field == wanted.field
                    && not.why.contains(&wanted.reason_holds)
            };
            if not_taken.len() == expected.len()
                && not_taken
                    .iter()
                    .zip(expected)
                    .all(|(not, wanted)| matches(not, wanted))
            {
                Vec::new()
            } else {
                vec![format!(
                    "{name}: refused {}, and the case expects {expected:?}",
                    Refused(not_taken)
                )]
            }
        }
    }
}

#[test]
fn the_crate_passes_every_inbox_case() {
    let cases = cases();
    assert!(!cases.is_empty(), "no cases under {}", folder().display());
    let differences: Vec<String> = cases.iter().flat_map(differences).collect();
    assert_eq!(differences, Vec::<String>::new());
}

#[test]
fn every_record_of_the_cases_passes_recfix() {
    for case in cases() {
        for under in ["start", "tip"] {
            for (path, text) in case.texts(under) {
                if path.ends_with(".rec") {
                    recfix(&text);
                }
            }
        }
    }
}

/// The clauses the cases cite, as numbers.
fn cited(cases: &[Case]) -> BTreeSet<i64> {
    cases
        .iter()
        .flat_map(|case| {
            case.written
                .satisfies
                .iter()
                .filter_map(toml::Value::as_integer)
        })
        .collect()
}

#[test]
fn the_cases_hold_the_inboxs_clauses() {
    let cited = cited(&cases());
    for clause in (25..=43).chain([63]) {
        assert!(cited.contains(&clause), "no inbox case cites {clause}");
    }
}

#[test]
fn the_cases_named_are_among_them() {
    let names: BTreeSet<String> = cases().into_iter().map(|case| case.name).collect();
    for name in [
        "a-reading-written-whole",
        "a-second-reading-refused",
        "an-excerpt-standing-nowhere-refused",
        "a-drop-with-its-reason",
        "an-answered-with-what-settled-it",
        "a-challenge-with-its-claim",
        "a-join-making-its-item",
        "an-attach-to-a-sent-item",
        "a-route-with-its-chore",
        "a-word-not-of-the-six-refused",
        "a-second-standing-move-refused",
        "a-replacement-naming-what-it-replaces",
        "a-batch-refused-whole",
        "a-proposed-batchs-moves-do-not-stand",
        "an-item-under-a-used-name-refused",
        "an-item-nesting-under-another-refused",
        "a-brief-rewritten-in-place",
        "a-send-keeps-the-brief",
        "import-lines-written-as-read",
        "an-imported-capture-held-by-key",
        "new-territory-read-as-a-join",
        "a-move-of-another-word-left-unmoved",
        "a-record-naming-no-format-refused",
        "a-file-the-layout-does-not-place-refused",
    ] {
        assert!(names.contains(name), "no case {name}");
    }
}

#[test]
fn a_case_of_a_second_standing_move() {
    let case = cases()
        .into_iter()
        .find(|case| case.name == "a-second-standing-move-refused")
        .unwrap();
    assert!(case.texts("start").contains_key("inbox/moves/moves.rec"));
    let refused = run(&case).unwrap_err();
    assert!(refused.0[0].why.contains("standing move"), "{refused}");
    assert_eq!(differences(&case), Vec::<String>::new());
}

#[test]
fn a_misspelt_field() {
    let files = BTreeMap::from([(
        "case.toml".to_string(),
        b"satisfies = [29]\n\n[operaton.move]\nsignal = \"release/acme/1.0#1\"\nword = \"drop\"\n"
            .to_vec(),
    )]);
    let refused = read_case("misspelt", files).err().expect("a refusal");
    assert!(refused.starts_with("misspelt: "), "{refused}");
    assert!(refused.contains("`operaton`"), "{refused}");
}

#[test]
fn a_case_the_crate_fails() {
    // A batch the case expects refused whole, which the crate takes.
    let mut case = cases()
        .into_iter()
        .find(|case| case.name == "a-proposed-batchs-moves-do-not-stand")
        .unwrap();
    case.files.retain(|path, _| !path.starts_with("tip/"));
    case.written.refused.push(ExpectedRefusal {
        what: "release/acme/1.0#1".into(),
        field: None,
        reason_holds: "standing move".into(),
    });
    let said = differences(&case);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with(
            "a-proposed-batchs-moves-do-not-stand: expects a refusal, and the write was taken"
        ),
        "{}",
        said[0]
    );
    assert!(
        said[0].contains("inbox/moves/batches/triage-1.rec"),
        "{}",
        said[0]
    );
}

#[test]
fn a_case_reads_only_the_files_of_its_folder() {
    // Copied whole to another folder, every case gives the same verdict.
    let copy = tempfile::tempdir().unwrap();
    for case in cases() {
        for (path, bytes) in &case.files {
            let file = copy.path().join(&case.name).join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, bytes).unwrap();
        }
        let mut files = BTreeMap::new();
        files_under(&copy.path().join(&case.name), "", &mut files);
        let copied = read_case(&case.name, files).unwrap();
        assert_eq!(differences(&copied), differences(&case), "{}", case.name);
    }
}
