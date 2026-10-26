//! A write made one commit, on the tip read, through a temporary index in
//! a folder removed when the commit is made, touching no working tree: its
//! tree the tip's with the write's files added or replaced and nothing else
//! changed, its message naming what it did and who asked (45, 47, 50; C.3
//! step 3; core.27, core.29).
//!
//! The author and committer are named by who asked, with an empty email,
//! and dated by the caller's clock, and the commit is never signed: a
//! commit needs no identity of the machine's, and two machines form it the
//! same way (core.1, core.14).

use std::collections::BTreeSet;
use std::path::Path;

use time::OffsetDateTime;

use crate::git::Git;
use crate::keeper::{Failure, Files, Step, Tip};

/// Why a write was not committed.
pub(crate) enum NotMade {
    /// A path of the write stands where the tip, or the write itself, holds
    /// a folder, or under a file: committing it would drop a file.
    Path {
        path: String,
        reason: String,
    },
    Failed(Failure),
}

impl From<Failure> for NotMade {
    fn from(failure: Failure) -> NotMade {
        NotMade::Failed(failure)
    }
}

/// The commit's message: what it did, a blank line, and who asked (50).
pub(crate) fn message(what: &str, who: &str) -> String {
    format!("{what}\n\nAsked-by: {who}\n")
}

/// `when` as git reads a date: seconds since the epoch and the offset.
fn git_date(when: OffsetDateTime) -> String {
    let offset = when.offset();
    let (hours, minutes, _) = offset.as_hms();
    let sign = if offset.is_negative() { '-' } else { '+' };
    format!(
        "@{} {sign}{:02}{:02}",
        when.unix_timestamp(),
        hours.unsigned_abs(),
        minutes.unsigned_abs()
    )
}

/// Makes `files` one commit on `tip` in `git_dir`, and gives the commit.
pub(crate) fn make(
    git_dir: &Path,
    tip: Option<&Tip>,
    files: &Files,
    what: &str,
    who: &str,
    when: OffsetDateTime,
) -> Result<String, NotMade> {
    let failed = |said: String| NotMade::Failed(Failure::new(Step::Commit, said));
    let folder = tempfile::Builder::new()
        .prefix("weir-index.")
        .tempdir_in(git_dir)
        .map_err(|error| {
            failed(format!(
                "no temporary index in {}: {error}",
                git_dir.display()
            ))
        })?;
    let index = folder.path().join("index");
    let index = index
        .to_str()
        .ok_or_else(|| failed(format!("the index {} is no UTF-8 path", index.display())))?;
    let date = git_date(when);
    let git = Git::new(git_dir)
        .env("GIT_INDEX_FILE", index)
        .env("GIT_AUTHOR_NAME", who)
        .env("GIT_AUTHOR_EMAIL", "")
        .env("GIT_AUTHOR_DATE", date.as_str())
        .env("GIT_COMMITTER_NAME", who)
        .env("GIT_COMMITTER_EMAIL", "")
        .env("GIT_COMMITTER_DATE", date.as_str());

    match tip {
        Some(tip) => git.text(&["read-tree", tip.commit()], None),
        None => git.text(&["read-tree", "--empty"], None),
    }
    .map_err(failed)?;
    let listed = git.run(&["ls-files", "-z"], None).map_err(failed)?;
    if !listed.ok() {
        return Err(failed(listed.said()));
    }
    let held: BTreeSet<String> = listed
        .out
        .split(|&byte| byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8_lossy(path).into_owned())
        .collect();
    check_places(&held, files)?;

    let mut entries = Vec::new();
    for (path, text) in files {
        let blob = git
            .text(&["hash-object", "-w", "--stdin"], Some(text.as_bytes()))
            .map_err(failed)?;
        entries.extend_from_slice(format!("100644 {blob}\t{path}\0").as_bytes());
    }
    git.text(
        &["update-index", "-z", "--add", "--index-info"],
        Some(&entries),
    )
    .map_err(failed)?;
    let tree = git.text(&["write-tree"], None).map_err(failed)?;
    let mut args = vec!["commit-tree", "--no-gpg-sign", tree.as_str()];
    if let Some(tip) = tip {
        args.extend(["-p", tip.commit()]);
    }
    let commit = git
        .text(&args, Some(message(what, who).as_bytes()))
        .map_err(failed)?;
    folder
        .close()
        .map_err(|error| failed(format!("the temporary index not removed: {error}")))?;
    Ok(commit)
}

/// Refuses a path of the write that would take a folder's place, or a
/// file's as its folder, among the tip's files and the write's own; git's
/// index would drop the file it displaced, and a write never removes one
/// (41).
fn check_places(held: &BTreeSet<String>, files: &Files) -> Result<(), NotMade> {
    let all: BTreeSet<&str> = held
        .iter()
        .map(String::as_str)
        .chain(files.keys().map(String::as_str))
        .collect();
    for path in files.keys() {
        let folder = format!("{path}/");
        if let Some(under) = all
            .range(folder.as_str()..)
            .next()
            .filter(|other| other.starts_with(&folder))
        {
            return Err(NotMade::Path {
                path: path.clone(),
                reason: format!("stands where a folder holds `{under}`"),
            });
        }
        let mut end = 0;
        while let Some(slash) = path[end..].find('/') {
            end += slash;
            let above = &path[..end];
            if all.contains(above) {
                return Err(NotMade::Path {
                    path: path.clone(),
                    reason: format!("lies under the file `{above}`"),
                });
            }
            end += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(paths: &[&str]) -> Files {
        paths
            .iter()
            .map(|path| (path.to_string(), String::new()))
            .collect()
    }

    fn held(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|path| path.to_string()).collect()
    }

    #[test]
    fn a_path_taking_a_folders_place_or_lying_under_a_file() {
        let tip = held(&["inbox/moves/moves.rec", "inbox/a"]);
        assert!(check_places(&tip, &files(&["inbox/moves/moves.rec", "inbox/b"])).is_ok());
        assert!(check_places(&tip, &files(&["inbox/moves.rec"])).is_ok());
        for path in [
            "inbox/moves",
            "inbox",
            "inbox/a/b",
            "inbox/moves/moves.rec/x",
        ] {
            assert!(check_places(&tip, &files(&[path])).is_err(), "{path} taken");
        }
        assert!(check_places(&held(&[]), &files(&["x", "x/y"])).is_err());
    }

    #[test]
    fn a_date_as_git_reads_it() {
        let when = OffsetDateTime::from_unix_timestamp(1_700_000_000)
            .unwrap()
            .to_offset(time::UtcOffset::from_hms(-5, -30, 0).unwrap());
        assert_eq!(git_date(when), "@1700000000 -0530");
        let when = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        assert_eq!(git_date(when), "@1700000000 +0000");
    }
}
