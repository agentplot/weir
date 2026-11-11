//! The tests' silent remote: one that takes the call and never answers,
//! reached with no network (68). Its address is an `ssh://` one, and the
//! cache's `core.sshCommand` a script that writes its process id to a file
//! in the test's folder and then becomes a `sleep 60`; `ssh.variant` is
//! `simple`, so git runs it once, with no probe.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::keeper::Branch;

/// The silent remote's address.
pub(crate) const URL: &str = "ssh://silent.invalid/inbox.git";

/// A bare cache whose remote never answers, in a folder of the test's.
pub(crate) struct Silent {
    folder: TempDir,
    cache: PathBuf,
}

impl Silent {
    pub(crate) fn new() -> Silent {
        let folder = tempfile::tempdir().expect("a temporary folder");
        let cache = folder.path().join("cache.git");
        let script = folder.path().join("silent.sh");
        let pid = folder.path().join("pid");
        fs::write(
            &script,
            format!("#!/bin/sh\necho $$ > '{}'\nexec sleep 60\n", pid.display()),
        )
        .expect("the silent remote's script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
            .expect("the script made runnable");
        let git = |args: &[&str]| {
            let ran = Command::new("git")
                .args(args)
                .stdin(Stdio::null())
                .output()
                .expect("git runs");
            assert!(ran.status.success(), "git {args:?}: {ran:?}");
        };
        let dir = cache.to_str().expect("a UTF-8 folder");
        git(&["init", "--quiet", "--bare", dir]);
        let command = format!("'{}'", script.display());
        git(&["--git-dir", dir, "config", "core.sshCommand", &command]);
        git(&["--git-dir", dir, "config", "ssh.variant", "simple"]);
        Silent { folder, cache }
    }

    /// The branch `inbox` on the silent remote, through the cache, its
    /// try's limit 300 ms.
    pub(crate) fn branch(&self) -> Branch {
        let mut branch = Branch::remote("inbox", &self.cache, URL);
        branch.limit = Duration::from_millis(300);
        branch
    }

    /// The cache's git directory.
    pub(crate) fn cache(&self) -> &std::path::Path {
        &self.cache
    }

    /// Whether the process the call started, which wrote its id, is gone
    /// within a second, as `kill -0` says.
    pub(crate) fn called_is_gone(&self) -> bool {
        let pid = fs::read_to_string(self.folder.path().join("pid"))
            .expect("the silent remote was called");
        let pid = pid.trim();
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            let there = Command::new("sh")
                .args(["-c", &format!("kill -0 {pid}")])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("sh runs")
                .success();
            if !there {
                return true;
            }
            if Instant::now() >= until {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
