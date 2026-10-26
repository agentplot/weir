//! The crate's code names no tool of the family (core.2).

use std::fs;
use std::path::Path;

/// The names core.2 rules out of every module but the signals-folder
/// import and the pack's cases.
const RULED_OUT: [&str; 8] = [
    "millpond",
    "flume",
    "sounding",
    "crew",
    "headwater",
    "eddy",
    "flywheel",
    "swancloud",
];

/// Every file under `dir`, with its path.
fn files(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).expect("a readable directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            files(&path, found);
        } else {
            let text = fs::read_to_string(&path).expect("a text file");
            found.push((path.display().to_string(), text));
        }
    }
}

#[test]
fn the_code_names_no_tool_of_the_family() {
    let mut found = Vec::new();
    files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    assert!(!found.is_empty(), "no files under src");
    let named: Vec<String> = found
        .iter()
        .flat_map(|(path, text)| {
            let text = text.to_lowercase();
            RULED_OUT
                .iter()
                .filter(move |name| text.contains(*name))
                .map(move |name| format!("{path}: {name}"))
        })
        .collect();
    assert_eq!(named, Vec::<String>::new());
}
