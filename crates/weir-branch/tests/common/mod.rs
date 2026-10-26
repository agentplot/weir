//! What the crate's tests share: bare repositories made in temporary
//! folders, a branch's files seeded through git's plumbing as another
//! machine's writer would land them, a commit read back, and GNU recutils'
//! `recfix`, git and recutils both from the devenv shell. A helper fails
//! when git or `recfix` is not on `PATH`.
//!
//! Every git command runs with the variables that would send it elsewhere
//! removed, as the crate's own do, so a test that sets them for the crate
//! still seeds and reads the repository it names.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const REDIRECTS: [&str; 7] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_QUARANTINE_PATH",
];

fn command(git_dir: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("--git-dir").arg(git_dir);
    for name in REDIRECTS {
        command.env_remove(name);
    }
    command.env("GIT_TERMINAL_PROMPT", "0").env("LC_ALL", "C");
    command
}

fn run(mut command: Command, input: Option<&[u8]>) -> Output {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .unwrap_or_else(|error| panic!("git is not on PATH: {error}"));
    if let Some(input) = input {
        child
            .stdin
            .take()
            .expect("git's standard input")
            .write_all(input)
            .expect("git's standard input written");
    }
    child.wait_with_output().expect("git ran")
}

/// Runs git in `git_dir` and gives what it printed, its last newline
/// dropped; fails the test with git's words when git fails.
pub fn git(git_dir: &Path, args: &[&str]) -> String {
    git_with(git_dir, args, &[], None)
}

/// [`git`] with variables set and text on standard input.
pub fn git_with(
    git_dir: &Path,
    args: &[&str],
    env: &[(&str, &str)],
    input: Option<&[u8]>,
) -> String {
    let mut command = command(git_dir);
    command.args(args);
    for (name, value) in env {
        command.env(name, value);
    }
    let output = run(command, input);
    assert!(
        output.status.success(),
        "git {args:?} in {} failed: {}",
        git_dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).expect("git prints UTF-8");
    printed.strip_suffix('\n').unwrap_or(&printed).to_string()
}

/// A bare repository at `parent/name`, made with `git init --bare`.
pub fn bare(parent: &Path, name: &str) -> PathBuf {
    let path = parent.join(name);
    std::fs::create_dir_all(&path).expect("the repository's folder");
    git(&path, &["init", "--quiet", "--bare"]);
    path
}

/// A repository with a working tree at `parent/name`, its branch `main`
/// checked out holding `files`, committed and in its index.
pub fn working(parent: &Path, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let tree = parent.join(name);
    std::fs::create_dir_all(&tree).expect("the working tree's folder");
    let git_dir = tree.join(".git");
    let mut init = Command::new("git");
    for name in REDIRECTS {
        init.env_remove(name);
    }
    init.args(["init", "--quiet", "--initial-branch", "main"])
        .arg(&tree);
    assert!(run(init, None).status.success(), "git init failed");
    for (path, text) in files {
        let path = tree.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let in_tree = |args: &[&str]| {
        let mut command = command(&git_dir);
        command
            .arg("--work-tree")
            .arg(&tree)
            .args(args)
            .env("GIT_AUTHOR_NAME", "tester")
            .env("GIT_AUTHOR_EMAIL", "")
            .env("GIT_COMMITTER_NAME", "tester")
            .env("GIT_COMMITTER_EMAIL", "");
        let output = run(command, None);
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    in_tree(&["add", "--all"]);
    in_tree(&["commit", "--quiet", "--no-gpg-sign", "-m", "start"]);
    git_dir
}

/// The commit `branch` stands at in `git_dir`, or none.
pub fn tip(git_dir: &Path, branch: &str) -> Option<String> {
    let mut command = command(git_dir);
    command.args([
        "rev-parse",
        "--verify",
        "--quiet",
        &format!("refs/heads/{branch}"),
    ]);
    let output = run(command, None);
    let printed = String::from_utf8(output.stdout).expect("git prints UTF-8");
    let commit = printed.trim();
    (output.status.success() && !commit.is_empty()).then(|| commit.to_string())
}

/// Lands `files` on `branch` in `git_dir` as one commit on its tip, or as
/// its first, through a temporary index and `update-ref` from the tip, its
/// message in the protocol's form: another machine's writer's write. Gives
/// the commit.
pub fn land(
    git_dir: &Path,
    branch: &str,
    files: &BTreeMap<String, String>,
    what: &str,
    who: &str,
) -> String {
    let entries: Vec<(&[u8], &[u8])> = files
        .iter()
        .map(|(path, text)| (path.as_bytes(), text.as_bytes()))
        .collect();
    land_raw(git_dir, branch, &entries, what, who)
}

/// [`land`] with paths and contents as bytes, which need not be UTF-8.
pub fn land_raw(
    git_dir: &Path,
    branch: &str,
    files: &[(&[u8], &[u8])],
    what: &str,
    who: &str,
) -> String {
    let parent = tip(git_dir, branch);
    let folder = tempfile::tempdir().expect("a temporary folder");
    let index = folder.path().join("index");
    let index = index.to_str().expect("a UTF-8 temporary path");
    let with_index = [("GIT_INDEX_FILE", index)];
    match &parent {
        Some(parent) => git_with(git_dir, &["read-tree", parent], &with_index, None),
        None => git_with(git_dir, &["read-tree", "--empty"], &with_index, None),
    };
    let mut entries = Vec::new();
    for (path, text) in files {
        let blob = git_with(git_dir, &["hash-object", "-w", "--stdin"], &[], Some(text));
        entries.extend_from_slice(format!("100644 {blob}\t").as_bytes());
        entries.extend_from_slice(path);
        entries.push(0);
    }
    git_with(
        git_dir,
        &["update-index", "-z", "--add", "--index-info"],
        &with_index,
        Some(&entries),
    );
    let tree = git_with(git_dir, &["write-tree"], &with_index, None);
    let mut args = vec!["commit-tree", "--no-gpg-sign", tree.as_str()];
    if let Some(parent) = &parent {
        args.extend(["-p", parent.as_str()]);
    }
    let message = format!("{what}\n\nAsked-by: {who}\n");
    let identity = [
        ("GIT_AUTHOR_NAME", who),
        ("GIT_AUTHOR_EMAIL", ""),
        ("GIT_COMMITTER_NAME", who),
        ("GIT_COMMITTER_EMAIL", ""),
    ];
    let commit = git_with(git_dir, &args, &identity, Some(message.as_bytes()));
    let zeros = "0".repeat(40);
    let old = parent.as_deref().unwrap_or(&zeros);
    git(
        git_dir,
        &["update-ref", &format!("refs/heads/{branch}"), &commit, old],
    );
    commit
}

/// [`land`] from `(path, text)` pairs.
pub fn seed(git_dir: &Path, branch: &str, files: &[(&str, &str)], what: &str, who: &str) -> String {
    let files = files
        .iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect();
    land(git_dir, branch, &files, what, who)
}

/// A commit's full message.
pub fn message(git_dir: &Path, commit: &str) -> String {
    let mut command = command(git_dir);
    command.args(["cat-file", "commit", commit]);
    let output = run(command, None);
    assert!(output.status.success(), "no commit {commit}");
    let text = String::from_utf8(output.stdout).expect("a UTF-8 commit");
    let (_, message) = text.split_once("\n\n").expect("a commit's message");
    message.to_string()
}

/// A commit's header line `name`, as `author` or `committer`.
pub fn header(git_dir: &Path, commit: &str, name: &str) -> String {
    let text = git(git_dir, &["cat-file", "commit", commit]);
    let (headers, _) = text.split_once("\n\n").expect("a commit's headers");
    headers
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name} ")))
        .unwrap_or_else(|| panic!("commit {commit} has no {name}"))
        .to_string()
}

/// A commit's parents, in order.
pub fn parents(git_dir: &Path, commit: &str) -> Vec<String> {
    let line = git(git_dir, &["rev-list", "--parents", "-n", "1", commit]);
    line.split(' ').skip(1).map(str::to_string).collect()
}

/// Every commit from `start` (left out: the root) to `commit`, oldest
/// first, `start` itself not among them.
pub fn commits_since(git_dir: &Path, start: Option<&str>, commit: &str) -> Vec<String> {
    let range = match start {
        Some(start) => format!("{start}..{commit}"),
        None => commit.to_string(),
    };
    let listed = git(git_dir, &["rev-list", "--reverse", &range]);
    listed.lines().map(str::to_string).collect()
}

/// Every file at `commit`, by path, with its text whole.
pub fn files(git_dir: &Path, commit: &str) -> BTreeMap<String, String> {
    let listed = git(git_dir, &["ls-tree", "-r", "--name-only", "-z", commit]);
    listed
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(|path| {
            let mut command = command(git_dir);
            command.args(["cat-file", "blob", &format!("{commit}:{path}")]);
            let output = run(command, None);
            assert!(output.status.success(), "no file {path} at {commit}");
            let text = String::from_utf8(output.stdout).expect("a UTF-8 file");
            (path.to_string(), text)
        })
        .collect()
}

/// Every ref in `git_dir` with the commit it names, as `<commit> <ref>`
/// lines.
pub fn refs(git_dir: &Path) -> String {
    git(
        git_dir,
        &["for-each-ref", "--format=%(objectname) %(refname)"],
    )
}

/// What `recfix --check` says of `text`: `None` when it takes it, its words
/// when it refuses it.
pub fn recfix_says(text: &str) -> Option<String> {
    let mut child = Command::new("recfix")
        .arg("--check")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("recfix is not on PATH: {error}"));
    child
        .stdin
        .take()
        .expect("recfix's standard input")
        .write_all(text.as_bytes())
        .expect("recfix's standard input written");
    let output = child.wait_with_output().expect("recfix ran");
    (!output.status.success()).then(|| String::from_utf8_lossy(&output.stderr).into_owned())
}

/// Holds `text` to `recfix`, as a write is held before it lands (core.24).
pub fn recfix(text: &str) {
    if let Some(said) = recfix_says(text) {
        panic!("recfix refused:\n{text}\n{said}");
    }
}

/// A clock standing at `time`, an RFC 3339 time.
pub fn at(time: &str) -> OffsetDateTime {
    OffsetDateTime::parse(time, &Rfc3339).expect("an RFC 3339 time")
}

/// The machine's clock, in UTC.
pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// A remote and a writer's cache in one temporary folder.
pub struct Stage {
    pub folder: tempfile::TempDir,
    pub remote: PathBuf,
}

impl Stage {
    pub fn new() -> Stage {
        let folder = tempfile::tempdir().expect("a temporary folder");
        let remote = bare(folder.path(), "remote.git");
        Stage { folder, remote }
    }

    /// The writer's cache: `cache` for the first machine, any other name
    /// for another.
    pub fn cache(&self, name: &str) -> PathBuf {
        self.folder.path().join(name)
    }

    /// The branch `inbox` on the remote, through the cache `name`.
    pub fn branch(&self, name: &str) -> weir_branch::Branch {
        weir_branch::Branch::remote(
            "inbox",
            self.cache(name),
            self.remote.to_str().expect("a UTF-8 path"),
        )
    }

    pub fn tip(&self) -> Option<String> {
        tip(&self.remote, "inbox")
    }
}

/// An ask of `what` by `who` with no bound set and a minute's stale age.
pub fn ask<'a>(
    what: &'a str,
    who: &'a str,
    clock: &'a dyn Fn() -> OffsetDateTime,
) -> weir_branch::Ask<'a> {
    weir_branch::Ask {
        what,
        who,
        bound: None,
        stale_after: std::time::Duration::from_secs(60),
        clock,
    }
}

/// `files` as a write's files.
pub fn write_of(files: &[(&str, &str)]) -> weir_branch::Files {
    files
        .iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect()
}

/// Every commit `git_dir` holds, by any ref.
pub fn all_commits(git_dir: &Path) -> Vec<String> {
    let listed = git(git_dir, &["rev-list", "--all"]);
    listed.lines().map(str::to_string).collect()
}

/// Every entry of `folder` whose name begins with `prefix`.
pub fn left_in(folder: &Path, prefix: &str) -> Vec<String> {
    std::fs::read_dir(folder)
        .expect("the folder")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(prefix))
        .collect()
}
