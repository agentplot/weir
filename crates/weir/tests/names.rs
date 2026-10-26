//! The command's own code names no tool of the family: what it says of a
//! body it imports comes from the crate that reads it (core.2).

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
fn the_commands_code_names_no_tool() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;
    for entry in fs::read_dir(&src).unwrap() {
        let file = entry.unwrap().path();
        let text = fs::read_to_string(&file).unwrap();
        assert!(
            named(&text).is_empty(),
            "{} names {:?}",
            file.display(),
            named(&text)
        );
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn a_tool_is_named_as_a_whole_word() {
    assert_eq!(named("the reader of crew's folders"), ["crew"]);
    assert!(named("a screw and soundings").is_empty());
}
