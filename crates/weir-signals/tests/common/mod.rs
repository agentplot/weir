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
