//! A command run to its limit in a process group of its own, its output
//! written to unnamed temporary files and read once it has ended, with no
//! thread (69; core.1, core.53).

use std::fs::File;
use std::io::{self, Read as _, Seek as _, SeekFrom};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::group::{end_group, is_there};
use crate::halt::Halt;
use crate::GRACE;

/// How often a running command is looked at.
const LOOK: Duration = Duration::from_millis(10);

/// What came first and ended a command's group: its limit or its caller's
/// halt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopped {
    /// Its limit came first.
    AtLimit,
    /// Its caller's halt came first.
    Halted,
}

/// How a command ended, and what it printed until then.
#[derive(Debug)]
pub struct Ended {
    /// Its leader's exit: by itself, or by the signal that ended its group;
    /// none only where no process ran.
    pub status: Option<ExitStatus>,
    /// What came first and ended its group, its limit or its caller's halt;
    /// none where it ended by itself.
    pub stopped: Option<Stopped>,
    /// What it printed on standard output, its first `kept` bytes.
    pub stdout: Vec<u8>,
    /// What it printed on standard error, its first `kept` bytes.
    pub stderr: Vec<u8>,
    /// Whether what was kept of its standard output is less than all it
    /// printed there.
    pub stdout_cut: bool,
    /// Whether what was kept of its standard error is less than all it
    /// printed there.
    pub stderr_cut: bool,
}

/// `command` run in a process group of its own to `limit` or `halt`,
/// whichever comes first, everything it printed kept, as [`output_kept`]. Its standard input is the caller's to set. `Err` is the
/// command failing to start, or a temporary file not made or read.
pub fn output(command: &mut Command, limit: Duration, halt: &Halt) -> io::Result<Ended> {
    output_kept(command, limit, u64::MAX, halt)
}

/// `command` run in a process group of its own to `limit`, the first
/// `kept` bytes of each of its standard output and error kept. The command
/// has ended once its leader has exited and no process of its group is
/// left. At the limit its group is asked to end, given [`GRACE`], then
/// killed, and the command is said stopped. Its standard input is the
/// caller's to set.
pub fn output_kept(
    command: &mut Command,
    limit: Duration,
    kept: u64,
    _halt: &Halt,
) -> io::Result<Ended> {
    let mut out = tempfile::tempfile()?;
    let mut err = tempfile::tempfile()?;
    let mut child = command
        .stdout(Stdio::from(out.try_clone()?))
        .stderr(Stdio::from(err.try_clone()?))
        .process_group(0)
        .spawn()?;
    let group = child.id() as libc::pid_t;
    let (status, stopped) = match wait(&mut child, group, limit) {
        Ok(waited) => waited,
        Err(error) => {
            end(&mut child, group, &mut None);
            return Err(error);
        }
    };
    let (stdout, stdout_cut) = taken(&mut out, kept)?;
    let (stderr, stderr_cut) = taken(&mut err, kept)?;
    Ok(Ended {
        status: Some(status),
        stopped,
        stdout,
        stderr,
        stdout_cut,
        stderr_cut,
    })
}

/// Waits for `child`, the leader of `group`, until it is reaped and its
/// group is gone, or to `limit`, when the group is ended. Gives the
/// leader's exit and what stopped the command, if anything did.
fn wait(
    child: &mut Child,
    group: libc::pid_t,
    limit: Duration,
) -> io::Result<(ExitStatus, Option<Stopped>)> {
    let until = Instant::now() + limit;
    let mut status = None;
    loop {
        if status.is_none() {
            status = child.try_wait()?;
        }
        if let Some(status) = status {
            if !is_there(group) {
                return Ok((status, None));
            }
        }
        if Instant::now() >= until {
            break;
        }
        std::thread::sleep(LOOK);
    }
    end(child, group, &mut status);
    let status = match status {
        Some(status) => status,
        None => child.wait()?,
    };
    Ok((status, Some(Stopped::AtLimit)))
}

/// Ends `group`, reaping its leader `child` into `status` as it goes.
fn end(child: &mut Child, group: libc::pid_t, status: &mut Option<ExitStatus>) {
    end_group(group, GRACE, &mut || {
        if status.is_none() {
            *status = child.try_wait().ok().flatten();
        }
        status.is_some()
    });
}

/// The first `kept` bytes of `file` from its start, and whether it holds
/// more.
fn taken(file: &mut File, kept: u64) -> io::Result<(Vec<u8>, bool)> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.by_ref().take(kept).read_to_end(&mut bytes)?;
    let cut = file.metadata()?.len() > bytes.len() as u64;
    Ok((bytes, cut))
}
