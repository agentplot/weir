//! Running git: every command as `git --git-dir <dir>`, with the variables
//! that would send it to another repository removed from its environment,
//! so a caller inside a hook or a worktree never redirects it; never waiting
//! on a prompt; and always in one language (C.3). Reaching a remote is
//! git's own, through its credential helpers: the crate passes no secret
//! (core.3).

use std::ffi::OsStr;
use std::io::{Seek, SeekFrom, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};

/// The variables that would point git at another repository, index or
/// object store than the one named.
const REDIRECTS: [&str; 7] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_QUARANTINE_PATH",
];

/// What a command printed, and how it ended.
pub(crate) struct Ran {
    pub code: Option<i32>,
    pub out: Vec<u8>,
    pub err: Vec<u8>,
}

impl Ran {
    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }

    /// What it printed on standard output, its last newline dropped.
    pub fn text(&self) -> String {
        let text = String::from_utf8_lossy(&self.out);
        text.strip_suffix('\n').unwrap_or(&text).to_string()
    }

    /// Its words: what it printed on standard error, then on standard
    /// output, trimmed.
    pub fn said(&self) -> String {
        let err = String::from_utf8_lossy(&self.err);
        let out = String::from_utf8_lossy(&self.out);
        let said = [err.trim(), out.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        match (said.is_empty(), self.code) {
            (false, _) => said,
            (true, Some(code)) => format!("exited {code}, saying nothing"),
            (true, None) => "ended by a signal, saying nothing".to_string(),
        }
    }
}

/// Runs `program` with `args`, `input` on its standard input. The input
/// is handed through an unnamed temporary file, so no command waits on a
/// pipe the caller has not read. `Err` is the program failing to start.
pub(crate) fn run_program<S: AsRef<OsStr>>(
    mut command: Command,
    args: &[S],
    input: Option<&[u8]>,
) -> Result<Ran, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    command.args(args);
    match input {
        Some(input) => {
            let mut file = tempfile::tempfile()
                .map_err(|error| format!("no temporary file for {program}'s input: {error}"))?;
            file.write_all(input)
                .and_then(|()| file.seek(SeekFrom::Start(0)).map(|_| ()))
                .map_err(|error| format!("{program}'s input not written: {error}"))?;
            command.stdin(Stdio::from(file));
        }
        None => {
            command.stdin(Stdio::null());
        }
    }
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("{program} could not be run: {error}"))?;
    Ok(Ran {
        code: output.status.code(),
        out: output.stdout,
        err: output.stderr,
    })
}

/// git in one git directory, with the environment its commands run in.
pub(crate) struct Git<'a> {
    dir: &'a Path,
    env: Vec<(&'static str, String)>,
}

impl<'a> Git<'a> {
    pub fn new(dir: &'a Path) -> Git<'a> {
        Git {
            dir,
            env: Vec::new(),
        }
    }

    /// Sets `name` for every command this runs: the temporary index, the
    /// commit's identity and date.
    pub fn env(mut self, name: &'static str, value: impl Into<String>) -> Git<'a> {
        self.env.push((name, value.into()));
        self
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command.arg("--git-dir").arg(self.dir);
        for name in REDIRECTS {
            command.env_remove(name);
        }
        command.env("GIT_TERMINAL_PROMPT", "0").env("LC_ALL", "C");
        for (name, value) in &self.env {
            command.env(name, value);
        }
        command
    }

    /// Runs `git --git-dir <dir> <args>`.
    pub fn run<S: AsRef<OsStr>>(&self, args: &[S], input: Option<&[u8]>) -> Result<Ran, String> {
        run_program(self.command(), args, input)
    }

    /// Runs it and gives what it printed, or its words when it fails.
    pub fn text<S: AsRef<OsStr>>(
        &self,
        args: &[S],
        input: Option<&[u8]>,
    ) -> Result<String, String> {
        let ran = self.run(args, input)?;
        if ran.ok() {
            Ok(ran.text())
        } else {
            Err(ran.said())
        }
    }
}
