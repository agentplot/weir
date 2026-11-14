//! The crate's own code names no tool of the family (core.2), starts no
//! thread and holds no static, so a halt is its caller's alone (core.1,
//! core.53).

use std::fs;
use std::path::{Path, PathBuf};

/// The strings core.2 rules out of every module but the signals-folder
/// import's and the pack's cases.
const RULED_OUT: [&str; 8] = [
    "millpond",
    "flume",
    "sounding",
    "crew",
    "headwater",
    "eddy",
    "boomdrive",
    "swancloud",
];

/// The calls that start a thread.
const THREADS: [&str; 2] = ["thread::spawn", "thread::Builder"];

/// The beginnings of a line, trimmed, that declare a static.
const STATICS: [&str; 3] = ["static ", "pub static ", "pub(crate) static "];

/// Every source file under the crate's `src`, with its text.
fn sources() -> Vec<(PathBuf, String)> {
    fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(folder).expect("the crate's sources") {
            let path = entry.expect("a source's entry").path();
            if path.is_dir() {
                walk(&path, found);
            } else {
                found.push(path);
            }
        }
    }
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    walk(&src, &mut found);
    assert!(!found.is_empty(), "no sources under {}", src.display());
    found
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("a UTF-8 source");
            (path, text)
        })
        .collect()
}

/// Whether `text` holds `word` as a whole word: no letter, digit or `_`
/// on either side.
fn holds_word(text: &str, word: &str) -> bool {
    let part = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !part(before) && !part(after)
    })
}

#[test]
fn the_crate_names_no_tool() {
    let mut named = Vec::new();
    for (path, text) in sources() {
        let text = text.to_lowercase();
        for name in RULED_OUT {
            if holds_word(&text, name) {
                named.push(format!("{} names `{name}`", path.display()));
            }
        }
    }
    assert!(named.is_empty(), "core.2: {}", named.join("; "));
}

#[test]
fn the_crate_starts_no_thread() {
    let mut started = Vec::new();
    for (path, text) in sources() {
        for call in THREADS {
            if text.contains(call) {
                started.push(format!("{} holds `{call}`", path.display()));
            }
        }
    }
    assert!(started.is_empty(), "core.1: {}", started.join("; "));
}

#[test]
fn the_crate_holds_no_static() {
    let mut held = Vec::new();
    for (path, text) in sources() {
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if STATICS.iter().any(|start| line.starts_with(start)) || line.contains("thread_local!")
            {
                held.push(format!("{}:{}: `{line}`", path.display(), number + 1));
            }
        }
    }
    assert!(held.is_empty(), "core.1: {}", held.join("; "));
}
