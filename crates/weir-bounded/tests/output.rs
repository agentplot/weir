//! A command run to its limit in a process group of its own, and a group
//! ended, asked first and then killed (69; core.1, core.53). Each command
//! is `sh`, `sleep` or `cat`, and reaches no network.

use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use weir_bounded::end_group;

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
