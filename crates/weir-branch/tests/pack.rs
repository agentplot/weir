//! The pack's protocol cases, `pack/branch/<case>/case.toml`, read and run
//! against the crate, held to the scenarios of shared-branch-cases (44 to
//! 50, 65, 67, 68; C.3, C.7; core.35, core.36).
//!
//! A case is data alone, so any implementation of the protocol runs it the
//! same way: a runner makes a bare repository standing for the remote, or
//! for a repository with no remote, in a temporary folder, seeds the
//! branch `inbox` with the case's start, places the lock the case names,
//! asks for each write in turn, lands each meanwhile write itself with
//! git's plumbing after the writer's read, and compares each outcome and
//! its tries, every commit after the start and the last tip's files with
//! what the case expects.

mod common;

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use weir_branch::{write, Ask, Branch, Files, Tip, WriteError, Written, LOCK};

use common::{bare, commits_since, land, message, now, parents, recfix_says, tip};

/// The clauses every protocol case together must cite (44 to 50).
const PROTOCOL: std::ops::RangeInclusive<u32> = 44..=50;

/// The cases the pack holds at least (shared-branch-cases).
const NAMED: [&str; 11] = [
    "lands-on-the-tip-read",
    "first-write-makes-the-branch",
    "two-writes-from-one-tip",
    "a-held-key-at-the-new-tip",
    "a-second-standing-move",
    "kept-moving-past-the-bound",
    "the-bound-left-out",
    "a-branch-with-no-remote",
    "recfix-refuses-a-record",
    "a-held-lock-let-go",
    "a-stale-lock-broken",
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    satisfies: Vec<u32>,
    remote: bool,
    bound: Option<u32>,
    stale_after_seconds: u64,
    start: Option<BTreeMap<String, String>>,
    lock: Option<StartLock>,
    write: Vec<CaseWrite>,
    expect: Expect,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartLock {
    holder: String,
    age_seconds: u64,
    let_go_after_ms: Option<u64>,
}

/// A write's form: an add or an append, exactly one.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Add {
    path: String,
    text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Append {
    path: String,
    text: String,
    refused_when: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseWrite {
    who: String,
    what: String,
    add: Option<Add>,
    append: Option<Append>,
    #[serde(default)]
    meanwhile: Vec<Meanwhile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Meanwhile {
    #[serde(rename = "try")]
    on_try: Option<u32>,
    #[serde(default)]
    every_try: bool,
    who: String,
    what: String,
    add: Option<Add>,
    append: Option<Append>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expect {
    write: Vec<ExpectWrite>,
    #[serde(default)]
    commit: Vec<ExpectCommit>,
    tip: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectWrite {
    outcome: String,
    tries: u32,
    reason_holds: Option<String>,
    broke: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectCommit {
    what: String,
    asked_by: String,
}

/// A form a write takes, from an add or an append.
enum Rule<'a> {
    Add(&'a Add),
    Append(&'a Append),
}

fn rule<'a>(
    add: &'a Option<Add>,
    append: &'a Option<Append>,
    named: &str,
) -> Result<Rule<'a>, String> {
    match (add, append) {
        (Some(add), None) => Ok(Rule::Add(add)),
        (None, Some(append)) => Ok(Rule::Append(append)),
        _ => Err(format!(
            "{named} holds neither or both of `add` and `append`"
        )),
    }
}

impl Rule<'_> {
    /// The write formed from the files at a tip, `file` reading one: an add
    /// writes nothing over a path the tip holds (a held key, 14); an append
    /// is refused naming the line the tip's text holds (a standing move,
    /// 29).
    fn form(&self, file: impl Fn(&str) -> Result<Option<String>, String>) -> Result<Files, String> {
        let mut files = Files::new();
        match self {
            Rule::Add(add) => {
                if file(&add.path)?.is_none() {
                    files.insert(add.path.clone(), add.text.clone());
                }
            }
            Rule::Append(append) => {
                let before = file(&append.path)?.unwrap_or_default();
                if let Some(line) = &append.refused_when {
                    if before.lines().any(|held| held == line) {
                        return Err(line.clone());
                    }
                }
                files.insert(append.path.clone(), format!("{before}{}", append.text));
            }
        }
        Ok(files)
    }
}

/// The folder of the pack's protocol cases.
fn pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack/branch")
}

/// Reads the case `name` from its text: every field one the format names,
/// `satisfies` never empty, and each write's form one of the two.
fn read_case(name: &str, text: &str) -> Result<Case, String> {
    let at = format!("pack/branch/{name}");
    let case: Case = toml::from_str(text).map_err(|error| format!("{at}: {error}"))?;
    if case.satisfies.is_empty() {
        return Err(format!("{at}: `satisfies` cites no clause"));
    }
    if case.write.is_empty() {
        return Err(format!("{at}: no write is asked for"));
    }
    if case.expect.write.len() != case.write.len() {
        return Err(format!(
            "{at}: {} writes and {} expected",
            case.write.len(),
            case.expect.write.len()
        ));
    }
    for (n, write) in case.write.iter().enumerate() {
        rule(&write.add, &write.append, &format!("{at}: write {}", n + 1))?;
        for meanwhile in &write.meanwhile {
            let named = format!("{at}: write {}'s meanwhile", n + 1);
            rule(&meanwhile.add, &meanwhile.append, &named)?;
            if meanwhile.on_try.is_some() == meanwhile.every_try {
                return Err(format!(
                    "{named} names neither or both of `try` and `every_try`"
                ));
            }
        }
    }
    for expected in &case.expect.write {
        if !["landed", "nothing", "refused", "kept-moving"].contains(&expected.outcome.as_str()) {
            return Err(format!(
                "{at}: the outcome `{}` is none of the four",
                expected.outcome
            ));
        }
    }
    Ok(case)
}

/// Reads every case under `folder`, by name.
fn read_cases(folder: &Path) -> Result<BTreeMap<String, Case>, String> {
    let mut cases = BTreeMap::new();
    for entry in fs::read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(path.join("case.toml"))
            .map_err(|error| format!("pack/branch/{name}: no case.toml: {error}"))?;
        cases.insert(name.clone(), read_case(&name, &text)?);
    }
    Ok(cases)
}

/// Every clause from 44 to 50 cited by some case, or those none cites.
fn uncited(cases: &BTreeMap<String, Case>) -> Vec<u32> {
    let cited: BTreeSet<u32> = cases
        .values()
        .flat_map(|case| case.satisfies.iter().copied())
        .collect();
    PROTOCOL.filter(|clause| !cited.contains(clause)).collect()
}

/// A write's outcome as a case names it: the outcome, its tries, its reason
/// and the holder of a stale lock it broke.
struct Seen {
    outcome: &'static str,
    tries: Option<u32>,
    reason: String,
    broke: Option<String>,
}

fn seen(result: &Result<Written, WriteError>) -> Seen {
    let broke = |broke: Option<&weir_branch::Broken>| broke.map(|broke| broke.holder.clone());
    match result {
        Ok(Written::Landed(landed)) => Seen {
            outcome: "landed",
            tries: Some(landed.tries),
            reason: String::new(),
            broke: broke(landed.broke.as_ref()),
        },
        Ok(Written::Nothing(nothing)) => Seen {
            outcome: "nothing",
            tries: Some(nothing.tries),
            reason: String::new(),
            broke: broke(nothing.broke.as_ref()),
        },
        Err(error) => {
            let (outcome, reason) = match error {
                WriteError::Refused { reason, .. } => ("refused", reason.clone()),
                WriteError::Recfix { path, said, .. } => ("refused", format!("{path}: {said}")),
                WriteError::Path { path, reason, .. } => ("refused", format!("{path} {reason}")),
                WriteError::KeptMoving { .. } => ("kept-moving", error.to_string()),
                WriteError::Asked { .. } | WriteError::Failed { .. } => {
                    ("failed", error.to_string())
                }
            };
            Seen {
                outcome,
                tries: error.tries(),
                reason,
                broke: broke(error.broke()),
            }
        }
    }
}

/// Runs `case` against the crate in a temporary folder: `Err` names the
/// case and every way its outcome differs from what it expects.
fn run_case(name: &str, case: &Case) -> Result<(), String> {
    let folder = tempfile::tempdir().map_err(|error| error.to_string())?;
    let repo = bare(
        folder.path(),
        if case.remote {
            "remote.git"
        } else {
            "repo.git"
        },
    );
    let start = case
        .start
        .as_ref()
        .map(|files| land(&repo, "inbox", files, "start", "the pack"));
    let branch = if case.remote {
        let url = repo.to_str().unwrap();
        // The remote is the bare repository in this case's folder, reached
        // by its path: nothing is fetched from or pushed to another place.
        assert!(Path::new(url).starts_with(folder.path()));
        Branch::remote("inbox", folder.path().join("cache"), url)
    } else {
        Branch::local("inbox", &repo)
    };
    let lock_path = branch.git_dir().join(LOCK);
    if let Some(lock) = &case.lock {
        fs::create_dir_all(branch.git_dir()).unwrap();
        let taken_at = (now() - Duration::from_secs(lock.age_seconds))
            .format(&Rfc3339)
            .unwrap();
        fs::write(
            &lock_path,
            format!("holder: {}\ntaken_at: {taken_at}\n", lock.holder),
        )
        .unwrap();
    }

    let mut differs = Vec::new();
    thread::scope(|scope| {
        if let Some(after) = case.lock.as_ref().and_then(|lock| lock.let_go_after_ms) {
            let lock_path = lock_path.clone();
            scope.spawn(move || {
                thread::sleep(Duration::from_millis(after));
                fs::remove_file(lock_path).unwrap();
            });
        }
        for (n, (asked, expected)) in case.write.iter().zip(&case.expect.write).enumerate() {
            let tried = Cell::new(0);
            let ask = Ask {
                what: &asked.what,
                who: &asked.who,
                bound: case.bound,
                stale_after: Duration::from_secs(case.stale_after_seconds),
                clock: &now,
            };
            let writer = rule(&asked.add, &asked.append, "").unwrap();
            let result = write(&branch, &ask, |at: Option<&Tip>| {
                tried.set(tried.get() + 1);
                for meanwhile in &asked.meanwhile {
                    if meanwhile.every_try || meanwhile.on_try == Some(tried.get()) {
                        land_meanwhile(&repo, meanwhile);
                    }
                }
                writer.form(|path| match at {
                    Some(at) => at.file(path).map_err(|error| error.to_string()),
                    None => Ok(None),
                })
            });
            let seen = seen(&result);
            let write = format!("write {}", n + 1);
            if seen.outcome != expected.outcome {
                differs.push(format!(
                    "{write}: outcome expected {}, made {} ({result:?})",
                    expected.outcome, seen.outcome
                ));
            }
            if seen.tries != Some(expected.tries) {
                differs.push(format!(
                    "{write}: tries expected {}, made {}",
                    expected.tries,
                    seen.tries
                        .map_or("none".to_string(), |tries| tries.to_string())
                ));
            }
            if let Some(holds) = &expected.reason_holds {
                if !seen.reason.contains(holds.as_str()) {
                    differs.push(format!(
                        "{write}: reason expected to hold `{holds}`, made `{}`",
                        seen.reason
                    ));
                }
            }
            if seen.broke != expected.broke {
                differs.push(format!(
                    "{write}: broke expected {:?}, made {:?}",
                    expected.broke, seen.broke
                ));
            }
        }
    });

    let last = tip(&repo, "inbox");
    let commits = match &last {
        Some(last) => commits_since(&repo, start.as_deref(), last),
        None => Vec::new(),
    };
    let made: Vec<(String, String)> = commits
        .iter()
        .map(|commit| {
            let text = message(&repo, commit);
            let (what, rest) = text.split_once("\n\n").unwrap_or((&text, ""));
            let asked_by = rest
                .trim_end()
                .strip_prefix("Asked-by: ")
                .unwrap_or(rest)
                .to_string();
            (what.to_string(), asked_by)
        })
        .collect();
    let expected: Vec<(String, String)> = case
        .expect
        .commit
        .iter()
        .map(|commit| (commit.what.clone(), commit.asked_by.clone()))
        .collect();
    if made != expected {
        differs.push(format!("commits expected {expected:?}, made {made:?}"));
    }
    let mut before = start.clone();
    for commit in &commits {
        let one = parents(&repo, commit);
        if one != before.iter().cloned().collect::<Vec<_>>() {
            differs.push(format!("commit {commit}'s parents {one:?}, not {before:?}"));
        }
        before = Some(commit.clone());
    }
    let files = last.as_ref().map(|last| common::files(&repo, last));
    if files != case.expect.tip {
        differs.push(format!(
            "last tip expected {:?}, made {files:?}",
            case.expect.tip
        ));
    }

    if differs.is_empty() {
        Ok(())
    } else {
        Err(format!("pack/branch/{name}: {}", differs.join("; ")))
    }
}

/// Lands another machine's write on the branch `inbox` of `repo`, formed by
/// its rule from the files at the branch's tip now; nothing lands when the
/// rule writes nothing or refuses.
fn land_meanwhile(repo: &Path, meanwhile: &Meanwhile) {
    let at = tip(repo, "inbox").map(|at| common::files(repo, &at));
    let rule = rule(&meanwhile.add, &meanwhile.append, "").unwrap();
    let formed = rule.form(|path| Ok(at.as_ref().and_then(|files| files.get(path).cloned())));
    if let Ok(files) = formed {
        if !files.is_empty() {
            land(repo, "inbox", &files, &meanwhile.what, &meanwhile.who);
        }
    }
}

// Reading the cases

#[test]
fn every_protocol_clause_is_cited() {
    let cases = read_cases(&pack()).unwrap();
    assert!(!cases.is_empty(), "no case under pack/branch/");
    assert!(cases.values().all(|case| !case.satisfies.is_empty()));
    assert_eq!(uncited(&cases), Vec::<u32>::new(), "clauses no case cites");

    let mut without = cases.clone();
    without.retain(|_, case| !case.satisfies.contains(&48));
    assert_eq!(uncited(&without), vec![48]);
}

#[test]
fn a_case_that_cites_nothing() {
    let folder = tempfile::tempdir().unwrap();
    let text = fs::read_to_string(pack().join("lands-on-the-tip-read/case.toml")).unwrap();
    let case = folder.path().join("cites-nothing");
    fs::create_dir(&case).unwrap();

    let without: String = text
        .lines()
        .filter(|line| !line.starts_with("satisfies"))
        .map(|line| format!("{line}\n"))
        .collect();
    fs::write(case.join("case.toml"), &without).unwrap();
    let refused = read_cases(folder.path()).unwrap_err();
    assert!(refused.contains("pack/branch/cites-nothing"), "{refused}");
    assert!(refused.contains("satisfies"), "{refused}");

    fs::write(case.join("case.toml"), format!("satisfies = []\n{without}")).unwrap();
    let refused = read_cases(folder.path()).unwrap_err();
    assert!(refused.contains("pack/branch/cites-nothing"), "{refused}");
    assert!(refused.contains("cites no clause"), "{refused}");
}

#[test]
fn a_case_read() {
    let text = fs::read_to_string(pack().join("lands-on-the-tip-read/case.toml")).unwrap();
    read_case("lands-on-the-tip-read", &text).unwrap();
    let refused = read_case("x", &format!("colour = \"blue\"\n{text}")).unwrap_err();
    assert!(refused.contains("pack/branch/x"), "{refused}");
    assert!(refused.contains("colour"), "{refused}");
    let refused = read_case(
        "x",
        &text.replacen("[[write]]\n", "[[write]]\nwhen = \"now\"\n", 1),
    )
    .unwrap_err();
    assert!(refused.contains("when"), "{refused}");
}

// Running the cases

#[test]
fn a_remote_case() {
    let text = fs::read_to_string(pack().join("lands-on-the-tip-read/case.toml")).unwrap();
    let case = read_case("lands-on-the-tip-read", &text).unwrap();
    assert!(case.remote);
    run_case("lands-on-the-tip-read", &case).unwrap();
}

#[test]
fn a_case_the_crate_fails() {
    let text = fs::read_to_string(pack().join("a-held-key-at-the-new-tip/case.toml")).unwrap();
    let mut case = read_case("a-held-key-at-the-new-tip", &text).unwrap();
    assert_eq!(case.expect.write[0].tries, 2);
    case.expect.write[0].tries = 3;
    let failed = run_case("a-held-key-at-the-new-tip", &case).unwrap_err();
    assert!(
        failed.contains("pack/branch/a-held-key-at-the-new-tip"),
        "{failed}"
    );
    assert!(failed.contains("tries expected 3, made 2"), "{failed}");
}

#[test]
fn every_case_passes() {
    let cases = read_cases(&pack()).unwrap();
    let failed: Vec<String> = cases
        .iter()
        .filter_map(|(name, case)| run_case(name, case).err())
        .collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn the_cases_named() {
    let cases = read_cases(&pack()).unwrap();
    for name in NAMED {
        assert!(cases.contains_key(name), "no case {name}");
    }
}

#[test]
fn every_record_a_case_holds_passes_recfix() {
    for (name, case) in read_cases(&pack()).unwrap() {
        let held = case.start.iter().chain(case.expect.tip.iter()).flatten();
        for (path, text) in held.filter(|(path, _)| path.ends_with(".rec")) {
            if let Some(said) = recfix_says(text) {
                panic!("pack/branch/{name}: {path}: recfix refused: {said}");
            }
        }
    }
}

#[test]
fn an_add_over_a_held_path() {
    let add = Add {
        path: "inbox/captures/x/capture.rec".to_string(),
        text: "format: weir-capture/1\n".to_string(),
    };
    let rule = Rule::Add(&add);
    assert_eq!(
        rule.form(|_| Ok(Some("held".to_string()))).unwrap(),
        Files::new()
    );
    assert_eq!(rule.form(|_| Ok(None)).unwrap().len(), 1);
}

#[test]
fn an_append_meeting_its_line() {
    let append = Append {
        path: "inbox/moves/moves.rec".to_string(),
        text: "\nformat: weir-move/1\nsignal: x#1\n".to_string(),
        refused_when: Some("signal: x#1".to_string()),
    };
    let rule = Rule::Append(&append);
    let held = "format: weir-move/1\nsignal: x#1\n".to_string();
    assert_eq!(
        rule.form(|_| Ok(Some(held.clone()))).unwrap_err(),
        "signal: x#1"
    );
    let other = "format: weir-move/1\nsignal: y#1\n".to_string();
    let formed = rule.form(|_| Ok(Some(other.clone()))).unwrap();
    assert_eq!(
        formed["inbox/moves/moves.rec"],
        format!("{other}{}", append.text)
    );
}
