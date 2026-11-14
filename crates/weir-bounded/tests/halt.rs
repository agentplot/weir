//! A caller's halt: one flag its clones share, and a command ended at the
//! first of its limit and its halt (record 0008 as amended 2026-11-12;
//! core.1, core.53). Each command is `sh`, and reaches no network. A halt
//! set mid-call is set from a thread the test makes, never the crate.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use weir_bounded::{output, Halt, Stopped};

#[test]
fn a_halt_is_one_flag_its_clones_share() {
    let first = Halt::default();
    let clone = first.clone();
    assert!(!first.halted(), "a halt made is set");
    clone.halt();
    clone.halt();
    let second = Halt::default();
    assert!(first.halted(), "the clone's halt did not reach the first");
    assert!(!second.halted(), "a second halt is set");
    assert_eq!(first, clone);
    assert_ne!(first, second);
}

/// Whether the process `pid` is gone within a second: `kill(pid, 0)`
/// failing with `ESRCH`.
fn gone_within_a_second(pid: libc::pid_t) -> bool {
    let until = Instant::now() + Duration::from_secs(1);
    loop {
        // SAFETY: kill reads its two integer arguments and touches no
        // memory of this process; signal 0 sends nothing and only asks.
        let asked = unsafe { libc::kill(pid, 0) };
        if asked != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn sh(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]).stdin(Stdio::null());
    command
}

/// `command` run to `limit`, its halt set from a thread of the test's
/// `after` the call begins, and how long the call took.
fn halted_after(
    command: &mut Command,
    limit: Duration,
    after: Duration,
) -> (weir_bounded::Ended, Duration) {
    let halt = Halt::default();
    std::thread::scope(|scope| {
        let setter = halt.clone();
        let started = Instant::now();
        scope.spawn(move || {
            std::thread::sleep(after);
            setter.halt();
        });
        let ended = output(command, limit, &halt).expect("sh runs");
        (ended, started.elapsed())
    })
}

#[test]
fn a_halt_ends_a_command_with_its_group() {
    let (ended, took) = halted_after(
        &mut sh("echo begun; sleep 60 >/dev/null 2>&1 & echo $!; sleep 60"),
        Duration::from_secs(10),
        Duration::from_millis(300),
    );
    assert!(took < Duration::from_secs(5), "took {took:?}");
    assert_eq!(ended.stopped, Some(Stopped::Halted), "{ended:?}");
    let printed = String::from_utf8_lossy(&ended.stdout);
    assert!(printed.starts_with("begun\n"), "{ended:?}");
    let pid: libc::pid_t = printed
        .lines()
        .nth(1)
        .expect("the background sleep's id")
        .trim()
        .parse()
        .expect("a process id");
    assert!(
        gone_within_a_second(pid),
        "the background sleep {pid} is left"
    );
}

#[test]
fn a_command_begun_with_its_halt_set_runs_no_process() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let written = folder.path().join("written");
    let mut command = sh("echo ran > \"$0\"");
    command.arg(&written);
    let halt = Halt::default();
    halt.halt();
    let started = Instant::now();
    let ended = output(&mut command, Duration::from_secs(10), &halt).expect("nothing runs");
    let took = started.elapsed();
    assert!(took < Duration::from_secs(1), "took {took:?}");
    assert_eq!(ended.stopped, Some(Stopped::Halted), "{ended:?}");
    assert!(ended.status.is_none(), "{ended:?}");
    assert!(
        ended.stdout.is_empty() && ended.stderr.is_empty(),
        "{ended:?}"
    );
    assert!(!ended.stdout_cut && !ended.stderr_cut, "{ended:?}");
    assert!(!written.exists(), "the command ran");
}

#[test]
fn a_halt_after_the_limit_changes_nothing() {
    let (ended, _) = halted_after(
        &mut sh("trap '' TERM; while :; do :; done"),
        Duration::from_millis(300),
        Duration::from_secs(1),
    );
    assert_eq!(ended.stopped, Some(Stopped::AtLimit), "{ended:?}");
}
