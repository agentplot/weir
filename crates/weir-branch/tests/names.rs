//! The crate's own code names no tool of the family and no source: only
//! the signals-folder import and the pack's cases may (core.2).

use std::fs;
use std::path::{Path, PathBuf};

/// The strings core.2 rules out of every module but those two.
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

fn sources(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).expect("the crate's sources") {
        let path = entry.expect("a source's entry").path();
        if path.is_dir() {
            sources(&path, found);
        } else {
            found.push(path);
        }
    }
}

#[test]
fn no_source_names_a_tool_of_the_family() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    sources(&src, &mut found);
    assert!(!found.is_empty(), "no sources under {}", src.display());
    let mut named = Vec::new();
    for path in found {
        let text = fs::read_to_string(&path)
            .expect("a UTF-8 source")
            .to_lowercase();
        for name in RULED_OUT {
            if text.contains(name) {
                named.push(format!("{} names `{name}`", path.display()));
            }
        }
    }
    assert!(named.is_empty(), "core.2: {}", named.join("; "));
}
