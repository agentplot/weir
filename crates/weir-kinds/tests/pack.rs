//! The pack's kinds cases, `pack/kinds/<case>/case.toml`, each a
//! declaration and the kinds it reads into or each field refused, read
//! against the crate, held to "The pack holds the kind format's cases"
//! (kinds; 51 to 58, 65, 67; C.4, C.7; core.36).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use weir_kinds::{read_elaboration, read_kinds, Declaration, End, Problem};

/// A case as its `case.toml` writes it: a field the format does not name
/// fails the read, naming it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    satisfies: Vec<toml::Value>,
    declaration: String,
    expect: Expect,
}

/// What the declaration reads into: its kinds and the sections beside them
/// returned unread, or each field refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expect {
    kinds: Option<BTreeMap<String, ExpectKind>>,
    #[serde(default)]
    unread: Vec<String>,
    refused: Option<Vec<ExpectRefusal>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectKind {
    version: u64,
    stages: Vec<ExpectStage>,
    /// The kind's sections the format does not name, returned unread.
    #[serde(default)]
    unread: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectStage {
    name: String,
    /// Its deliverable as the format spells it, or `conversation`.
    end: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectRefusal {
    field: String,
    reason_holds: String,
}

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack/kinds")
}

/// Reads the case `name` from `text`, or names the case and why not.
fn read_case(name: &str, text: &str) -> Result<Case, String> {
    let case: Case =
        toml::from_str(text).map_err(|error| format!("{name}: {}", error.message()))?;
    if case.satisfies.is_empty() {
        return Err(format!("{name}: `satisfies` cites no clause"));
    }
    if case.expect.kinds.is_some() == case.expect.refused.is_some() {
        return Err(format!(
            "{name}: `expect` names its kinds or what is refused, exactly one of them"
        ));
    }
    Ok(case)
}

/// Every case of the pack, by its folder's name.
fn cases() -> Vec<(String, Case)> {
    let mut names: Vec<String> = fs::read_dir(folder())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let text = fs::read_to_string(folder().join(&name).join("case.toml"))
                .unwrap_or_else(|error| panic!("{name}: case.toml: {error}"));
            let case = read_case(&name, &text).unwrap_or_else(|why| panic!("{why}"));
            (name, case)
        })
        .collect()
}

/// A stage's end as the case spells it.
fn spelled(end: &End) -> String {
    match end {
        End::Deliverable(deliverable) => deliverable.to_string(),
        End::Conversation => "conversation".to_string(),
    }
}

/// The declaration read, its elaboration sections with it, as whoever runs
/// elaborations reads it: every problem of either, or the declaration.
fn read(text: &str) -> Result<Declaration, Vec<Problem>> {
    let declaration = read_kinds(text).map_err(|refusal| refusal.problems)?;
    let problems: Vec<Problem> = declaration
        .kinds
        .values()
        .filter_map(|kind| read_elaboration(kind).err())
        .flat_map(|refusal| refusal.problems)
        .collect();
    if problems.is_empty() {
        Ok(declaration)
    } else {
        Err(problems)
    }
}

/// How the crate's reading of the case differs from what it expects, each
/// difference naming the case; none when they agree.
fn differences(name: &str, case: &Case) -> Vec<String> {
    let read = read(&case.declaration);
    if read != self::read(&case.declaration) {
        return vec![format!("{name}: the declaration read twice reads two ways")];
    }
    match (read, &case.expect) {
        (
            Ok(declaration),
            Expect {
                kinds: Some(kinds),
                unread,
                ..
            },
        ) => {
            let mut said = Vec::new();
            let beside: Vec<&String> = declaration.sections.keys().collect();
            if beside != unread.iter().collect::<Vec<_>>() {
                said.push(format!(
                    "{name}: returns {beside:?} unread beside the kinds, and the case expects {unread:?}"
                ));
            }
            let read_names: BTreeSet<&String> = declaration.kinds.keys().collect();
            let expected_names: BTreeSet<&String> = kinds.keys().collect();
            if read_names != expected_names {
                said.push(format!(
                    "{name}: reads the kinds {read_names:?}, and the case expects {expected_names:?}"
                ));
            }
            for (kind_name, expected) in kinds {
                let Some(kind) = declaration.kinds.get(kind_name) else {
                    continue;
                };
                let stages: Vec<(String, String)> = kind
                    .stages
                    .iter()
                    .map(|stage| (stage.name.clone(), spelled(&stage.end)))
                    .collect();
                let expected_stages: Vec<(String, String)> = expected
                    .stages
                    .iter()
                    .map(|stage| (stage.name.clone(), stage.end.clone()))
                    .collect();
                if kind.version != expected.version || stages != expected_stages {
                    said.push(format!(
                        "{name}: reads {kind_name}@{} with {stages:?}, and the case expects \
                         {kind_name}@{} with {expected_stages:?}",
                        kind.version, expected.version
                    ));
                }
                let unread: Vec<&String> = kind.sections.keys().collect();
                if unread != expected.unread.iter().collect::<Vec<_>>() {
                    said.push(format!(
                        "{name}: returns {unread:?} of {kind_name} unread, and the case expects {:?}",
                        expected.unread
                    ));
                }
            }
            said
        }
        (
            Err(problems),
            Expect {
                refused: Some(refused),
                ..
            },
        ) => {
            let agrees = problems.len() == refused.len()
                && problems.iter().zip(refused).all(|(problem, expected)| {
                    problem.field == expected.field
                        && problem.reason.contains(&expected.reason_holds)
                });
            if agrees {
                Vec::new()
            } else {
                vec![format!(
                    "{name}: refuses {:?}, and the case expects {refused:?}",
                    problems.iter().map(ToString::to_string).collect::<Vec<_>>()
                )]
            }
        }
        (Ok(declaration), _) => vec![format!(
            "{name}: reads the kinds {:?}, and the case expects a refusal",
            declaration.kinds.keys().collect::<Vec<_>>()
        )],
        (Err(problems), _) => vec![format!(
            "{name}: refuses {:?}, and the case expects its kinds",
            problems.iter().map(ToString::to_string).collect::<Vec<_>>()
        )],
    }
}

#[test]
fn every_kinds_case_is_answered_as_it_says() {
    let cases = cases();
    assert!(!cases.is_empty(), "no cases under {}", folder().display());
    let differences: Vec<String> = cases
        .iter()
        .flat_map(|(name, case)| differences(name, case))
        .collect();
    assert_eq!(differences, Vec::<String>::new());
}

#[test]
fn the_cases_cite_each_of_51_to_58() {
    let cited: BTreeSet<i64> = cases()
        .iter()
        .flat_map(|(_, case)| case.satisfies.iter().filter_map(toml::Value::as_integer))
        .collect();
    for clause in 51..=58 {
        assert!(cited.contains(&clause), "no kinds case cites {clause}");
    }
}

#[test]
fn the_cases_of_s10_s11_and_s12() {
    let cases: BTreeMap<String, Case> = cases().into_iter().collect();
    let flume = &cases["flumes-own-declaration-read-twice-the-same"];
    let kinds = flume.expect.kinds.as_ref().unwrap();
    assert_eq!(kinds.keys().collect::<Vec<_>>(), ["change", "fix"]);
    let elaboration = &cases["an-elaboration-kind"];
    assert_eq!(
        elaboration.expect.kinds.as_ref().unwrap()["writeback"].unread,
        ["elaboration"]
    );
    for name in ["a-stage-with-two-deliverables", "a-kind-with-no-stage"] {
        assert_eq!(
            cases[name].expect.refused.as_ref().unwrap().len(),
            1,
            "{name}"
        );
    }
}

#[test]
fn a_stage_with_two_deliverables() {
    let (name, case) = cases()
        .into_iter()
        .find(|(name, _)| name == "a-stage-with-two-deliverables")
        .unwrap();
    assert!(case
        .declaration
        .contains(r#"deliverables = ["commits", "tasks"]"#));
    let problems = read(&case.declaration).unwrap_err();
    assert_eq!(problems[0].field, "kinds.bump.stages[0].deliverables");
    assert_eq!(differences(&name, &case), Vec::<String>::new());
}

#[test]
fn a_misspelt_field_of_the_case() {
    let refused = read_case(
        "misspelt",
        "satisfies = [57]\ndeclaration = \"\"\n\n[kind.fix]\nversion = 1\n",
    )
    .expect_err("a refusal");
    assert!(refused.starts_with("misspelt: "), "{refused}");
    assert!(refused.contains("`kind`"), "{refused}");
}
