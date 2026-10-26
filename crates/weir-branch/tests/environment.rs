//! git run with the variables that would send it elsewhere set reads and
//! writes only the git directory the crate names. The variables are the
//! process's, so this binary holds one test alone.

mod common;

use std::fs;
use std::path::Path;

use common::{ask, bare, now, refs, seed, tip, write_of};
use weir_branch::{read, write, Branch, Written};

/// Every file under `folder`, with its bytes.
fn snapshot(folder: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    let mut stack = vec![folder.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path).unwrap() {
                stack.push(entry.unwrap().path());
            }
        } else {
            found.push((path.display().to_string(), fs::read(&path).unwrap()));
        }
    }
    found.sort();
    found
}

#[test]
fn git_reads_and_writes_only_the_git_directory_named() {
    let folder = tempfile::tempdir().unwrap();
    let decoy = bare(folder.path(), "decoy.git");
    seed(&decoy, "inbox", &[("decoy", "decoy\n")], "decoy", "nobody");
    let remote = bare(folder.path(), "remote.git");
    let a = seed(&remote, "inbox", &[("a", "a\n")], "start", "operator@one");
    let local = bare(folder.path(), "local.git");
    let b = seed(&local, "inbox", &[("b", "b\n")], "start", "operator@one");
    let decoy_before = snapshot(&decoy);
    let index = folder.path().join("decoy-index");

    std::env::set_var("GIT_DIR", &decoy);
    std::env::set_var("GIT_INDEX_FILE", &index);
    std::env::set_var("GIT_WORK_TREE", folder.path());

    let cache = folder.path().join("cache");
    let remote_branch = Branch::remote("inbox", &cache, remote.to_str().unwrap());
    let at = read(&remote_branch).unwrap().unwrap();
    assert_eq!(at.commit(), a);
    assert_eq!(at.file("a").unwrap().as_deref(), Some("a\n"));
    assert_eq!(refs(&cache), format!("{a} refs/weir/inbox"));

    let at = read(&Branch::local("inbox", &local)).unwrap().unwrap();
    assert_eq!(at.commit(), b);
    assert_eq!(at.files("").unwrap().len(), 1);

    for branch in [remote_branch, Branch::local("inbox", &local)] {
        let written = write(&branch, &ask("add c", "operator@one", &now), |_| {
            Ok(write_of(&[("c", "c\n")]))
        })
        .unwrap();
        let Written::Landed(landed) = written else {
            panic!("not landed: {written:?}");
        };
        let repo = if branch.git_dir() == cache {
            &remote
        } else {
            &local
        };
        assert_eq!(tip(repo, "inbox"), Some(landed.commit));
    }

    assert_eq!(snapshot(&decoy), decoy_before, "the decoy changed");
    assert!(!index.exists(), "an index was written at GIT_INDEX_FILE");
}
