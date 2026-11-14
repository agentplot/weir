//! A command run to its limit in a process group of its own, and a group
//! ended, asked first and then killed (69; core.1, core.53). Each command
//! is `sh`, `sleep` or `cat`, and reaches no network.

use std::io::{Seek as _, SeekFrom, Write as _};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use weir_bounded::{end_group, output, output_kept, Halt, Stopped, GRACE};

#[test]
fn a_group_already_gone_is_left_alone() {
    let mut child = Command::new("sh")
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .process_group(0)
        .spawn()
        .expect("sh starts");
    child.wait().expect("sh is waited on");
    let started = Instant::now();
    end_group(child.id() as i32, Duration::from_secs(10), &mut || true);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "end_group took {:?}",
        started.elapsed()
    );
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

#[test]
fn a_process_that_never_ends_is_ended_with_its_group() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let pid_file = folder.path().join("pid");
    let mut command = sh("echo begun; echo said >&2; sleep 60 & echo $! > \"$0\"; sleep 60");
    command.arg(&pid_file);
    let started = Instant::now();
    let ended =
        output(&mut command, Duration::from_millis(300), &Halt::default()).expect("sh runs");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "took {:?}",
        started.elapsed()
    );
    assert_eq!(ended.stopped, Some(Stopped::AtLimit), "{ended:?}");
    assert_eq!(ended.stdout, b"begun\n");
    assert_eq!(ended.stderr, b"said\n");
    let pid: libc::pid_t = std::fs::read_to_string(&pid_file)
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
fn a_process_asked_to_end_says_so_in_its_grace() {
    let ended = output(
        &mut sh("trap 'echo asked; exit 0' TERM; sleep 60 & wait"),
        Duration::from_millis(300),
        &Halt::default(),
    )
    .expect("sh runs");
    assert_eq!(ended.stopped, Some(Stopped::AtLimit), "{ended:?}");
    assert_eq!(ended.stdout, b"asked\n");
}

#[test]
fn a_process_that_will_not_end_is_killed() {
    let started = Instant::now();
    let ended = output(
        &mut sh("trap '' TERM; while :; do :; done"),
        Duration::from_millis(300),
        &Halt::default(),
    )
    .expect("sh runs");
    let took = started.elapsed();
    assert_eq!(ended.stopped, Some(Stopped::AtLimit), "{ended:?}");
    assert!(took >= GRACE, "took {took:?}, less than the grace");
    assert!(took < Duration::from_secs(5), "took {took:?}");
}

#[test]
fn a_child_that_outlives_its_leader_is_ended_at_the_limit() {
    let ended = output(
        &mut sh("sleep 60 >/dev/null 2>&1 & echo $!; exit 0"),
        Duration::from_millis(300),
        &Halt::default(),
    )
    .expect("sh runs");
    assert_eq!(ended.stopped, Some(Stopped::AtLimit), "{ended:?}");
    assert_eq!(ended.status.and_then(|status| status.code()), Some(0));
    let pid: libc::pid_t = String::from_utf8_lossy(&ended.stdout)
        .trim()
        .parse()
        .expect("the background sleep's id");
    assert!(
        gone_within_a_second(pid),
        "the background sleep {pid} is left"
    );
}

#[test]
fn a_process_that_ends_gives_its_exit_and_all_it_printed() {
    let ended = output(
        &mut sh("echo out; echo err >&2; exit 3"),
        Duration::from_secs(10),
        &Halt::default(),
    )
    .expect("sh runs");
    assert_eq!(ended.stopped, None, "{ended:?}");
    assert_eq!(ended.status.and_then(|status| status.code()), Some(3));
    assert_eq!(ended.stdout, b"out\n");
    assert_eq!(ended.stderr, b"err\n");
    assert!(!ended.stdout_cut && !ended.stderr_cut, "{ended:?}");
}

#[test]
fn the_input_the_caller_sets_is_read() {
    let mut input = tempfile::tempfile().expect("a temporary file");
    input.write_all(b"asked").expect("the input written");
    input.seek(SeekFrom::Start(0)).expect("the input rewound");
    let ended = output(
        Command::new("cat").stdin(Stdio::from(input)),
        Duration::from_secs(10),
        &Halt::default(),
    )
    .expect("cat runs");
    assert_eq!(ended.stopped, None, "{ended:?}");
    assert_eq!(ended.stdout, b"asked");
}

#[test]
fn what_passes_the_kept_part_is_said_cut() {
    let ended = output_kept(
        &mut sh("printf 'whole\\npart' >&2; printf 'out'"),
        Duration::from_secs(10),
        8,
        &Halt::default(),
    )
    .expect("sh runs");
    assert_eq!(ended.stopped, None, "{ended:?}");
    assert_eq!(ended.stderr, b"whole\npa");
    assert!(ended.stderr_cut);
    assert_eq!(ended.stdout, b"out");
    assert!(!ended.stdout_cut);
}
