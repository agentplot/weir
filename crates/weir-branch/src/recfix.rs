//! `recfix` on every record file a write adds or replaces, before the write
//! is committed: the file's new text on its standard input, its refusal
//! reported with the file's path in place of `stdin` (46; C.2, C.3 step 2;
//! core.24).

use std::process::Command;

use crate::git::run_program;
use crate::keeper::{Failure, Step};

/// Whether `path` is a record file `recfix` holds a write to.
pub(crate) fn is_record(path: &str) -> bool {
    path.ends_with(".rec")
}

/// What `recfix --check` says of `text`, the new text of `path`: `None`
/// when it takes it, its words when it refuses it. `recfix` missing from
/// the path is a failure.
pub(crate) fn check(path: &str, text: &str) -> Result<Option<String>, Failure> {
    let ran = run_program(Command::new("recfix"), &["--check"], Some(text.as_bytes()))
        .map_err(|said| Failure::new(Step::Check, said))?;
    if ran.ok() {
        return Ok(None);
    }
    let said = ran
        .said()
        .lines()
        .map(|line| match line.strip_prefix("stdin:") {
            Some(rest) => format!("{path}:{rest}"),
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Some(said))
}
