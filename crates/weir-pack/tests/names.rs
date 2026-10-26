//! The crate's code names no tool of the family: only the pack's cases may
//! (core.2).

use std::fs;
use std::path::Path;

/// The names core.2 rules out, each read as a whole word in any case.
const TOOLS: [&str; 8] = [
    "millpond",
    "flume",
    "sounding",
    "crew",
    "headwater",
    "eddy",
    "flywheel",
    "swancloud",
];

/// The tools `text` names as whole words.
fn named(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    TOOLS
        .into_iter()
        .filter(|tool| {
            lower.match_indices(tool).any(|(at, _)| {
                let before = lower[..at].chars().next_back();
                let after = lower[at + tool.len()..].chars().next();
                !before.is_some_and(char::is_alphanumeric)
                    && !after.is_some_and(char::is_alphanumeric)
            })
        })
        .collect()
}

/// Every file under `dir`, with its text.
fn files(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, found);
        } else {
            let text = fs::read_to_string(&path).unwrap();
            found.push((path.display().to_string(), text));
        }
    }
}

#[test]
fn the_crates_code_names_no_tool() {
    let mut found = Vec::new();
    files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    assert!(!found.is_empty(), "no files under src");
    let naming: Vec<String> = found
        .iter()
        .filter(|(_, text)| !named(text).is_empty())
        .map(|(path, text)| format!("{path} names {:?}", named(text)))
        .collect();
    assert_eq!(naming, Vec::<String>::new());
}

#[test]
fn a_tool_is_named_as_a_whole_word() {
    assert_eq!(named("a case of Flume's kinds"), ["flume"]);
    assert!(named("a screw and soundings").is_empty());
}
