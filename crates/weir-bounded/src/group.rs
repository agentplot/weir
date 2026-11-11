//! A process group's ending: asked, given its grace, then killed (core.53).

use std::io;
use std::time::{Duration, Instant};

/// How often a group asked to end is looked at within its grace.
const ASKED_LOOK: Duration = Duration::from_millis(50);

/// How often a killed group is looked at, and for how long at most.
const KILLED_LOOK: Duration = Duration::from_millis(20);
const KILLED_FOR: Duration = Duration::from_secs(5);

/// Ends the process group `group`: asks it to end, waits up to `grace`
/// for it to go, then kills what is left and waits up to five seconds for
/// it to go. Before each look `reap` is called, which reaps the group's
/// leader, whose parent the caller is, since a leader left a zombie keeps
/// its group there. Returns at once when no process of the group is left.
pub fn end_group(group: i32, grace: Duration, reap: &mut dyn FnMut() -> bool) {
    let mut there = || {
        reap();
        is_there(group)
    };
    if !there() {
        return;
    }
    signal(group, libc::SIGTERM);
    let until = Instant::now() + grace;
    while Instant::now() < until {
        if !there() {
            return;
        }
        std::thread::sleep(ASKED_LOOK);
    }
    signal(group, libc::SIGKILL);
    let until = Instant::now() + KILLED_FOR;
    while there() && Instant::now() < until {
        std::thread::sleep(KILLED_LOOK);
    }
}

/// Whether any process of `group` is left: `killpg(group, 0)` succeeds,
/// or fails with anything but `ESRCH`, as for a process that may not be
/// signalled.
pub(crate) fn is_there(group: i32) -> bool {
    // SAFETY: killpg reads its two integer arguments and touches no memory
    // of this process; signal 0 sends nothing and only asks.
    let asked = unsafe { libc::killpg(group, 0) };
    asked == 0 || io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

/// Sends `signal` to every process of `group`, whatever it answers: a
/// group gone meanwhile is what was asked for.
fn signal(group: i32, signal: libc::c_int) {
    // SAFETY: killpg reads its two integer arguments and touches no memory
    // of this process.
    unsafe {
        libc::killpg(group, signal);
    }
}
