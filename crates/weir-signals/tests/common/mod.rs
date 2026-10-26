//! What the crate's tests share: GNU recutils, from the devenv shell, run
//! over every file the crate's operations form. A test that needs it fails
//! when it is not on `PATH`.

#![allow(dead_code)]

use std::io::Write as _;
use std::process::{Command, Output};

/// `program` run over `text`, written to a file of its own.
pub fn run(program: &str, args: &[&str], text: &str) -> Output {
    let mut file = tempfile::NamedTempFile::new().expect("a temporary file");
    file.write_all(text.as_bytes())
        .expect("the records written");
    file.flush().expect("the records flushed");
    let path = file.path().to_str().expect("a UTF-8 temporary path");
    Command::new(program)
        .args(args)
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("{program} is not on PATH: {error}"))
}

/// Holds `text` to `recfix`, as a write is held before it lands (core.24).
pub fn recfix(text: &str) {
    let output = run("recfix", &[], text);
    assert!(
        output.status.success(),
        "recfix refused:\n{text}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The value GNU recutils reads for `name` in the one record of `text`.
pub fn recsel_value(text: &str, name: &str) -> String {
    let output = run("recsel", &["-P", name], text);
    assert!(
        output.status.success(),
        "recsel refused:\n{text}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).expect("recsel prints UTF-8");
    printed.strip_suffix('\n').unwrap_or(&printed).to_string()
}

use weir_capture::{CameIn, Capture, Key};
use weir_signals::{InboxAt, Write};

/// The key `text`, which the test expects the rule to take.
pub fn key(text: &str) -> Key {
    Key::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// A capture under `key` from a run of the release adapter, unread.
pub fn capture(key_text: &str) -> Capture {
    Capture {
        key: key(key_text),
        source: key_text.split('/').next().unwrap_or("release").into(),
        event_at: "2026-10-10T14:02:11Z".into(),
        title: Some(format!("{key_text}, the title")),
        url: None,
        pointer: Some(format!("deneb:/data/inbox/{key_text}")),
        came_in: CameIn::Run("releases".into()),
        machine: "deneb".into(),
        taken_at: "2026-10-17T04:30:05Z".into(),
        unread: true,
    }
}

/// The inbox a tip of `files` holds, which the test expects to read.
pub fn read(files: &[(&str, &str)]) -> InboxAt {
    InboxAt::read(files.iter().copied()).unwrap_or_else(|error| panic!("{error}"))
}

/// Holds every `.rec` file of `write` to `recfix`, file by file (core.24).
pub fn recfix_write(write: &Write) {
    for (path, text) in &write.files {
        if path.ends_with(".rec") {
            recfix(text);
        }
    }
}

/// The inbox after `write`, which the test expects the check to take.
pub fn after(inbox: &InboxAt, write: &Write) -> InboxAt {
    recfix_write(write);
    inbox
        .check(write)
        .unwrap_or_else(|refused| panic!("{refused}"))
}

/// The paths `write` adds or replaces, in order.
pub fn paths(write: &Write) -> Vec<&str> {
    write.files.keys().map(String::as_str).collect()
}
