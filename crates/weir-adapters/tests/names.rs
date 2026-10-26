//! The signals-folder import is the one module of the crate that names a
//! tool of the family; every other file of `src/` names none (core.2).

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

#[test]
fn only_the_signals_folder_import_names_a_tool() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files: Vec<_> = fs::read_dir(&src)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    assert!(files.len() > 1, "{files:?}");
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let names = named(&text);
        if file.file_name().unwrap() == "signals_folder.rs" {
            assert!(names.contains(&"crew"), "{}", file.display());
        } else {
            assert!(names.is_empty(), "{} names {names:?}", file.display());
        }
    }
}

#[test]
fn a_tool_is_named_as_a_whole_word() {
    assert_eq!(named("the reader of crew's folders"), ["crew"]);
    assert_eq!(named("Flywheel later"), ["flywheel"]);
    assert!(named("a screw, a crewel and soundings").is_empty());
}
