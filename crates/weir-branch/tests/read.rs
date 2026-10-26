//! Reading a branch at one tip, held to the scenarios of shared-branch-write
//! ("A branch is read at one tip") on a remote and on a branch with no
//! remote (78, 80; I7).

mod common;

use std::path::Path;

use common::{bare, land_raw, refs, seed, tip};
use weir_branch::{read, Branch, Step};

fn remote_branch(folder: &Path, remote: &Path) -> Branch {
    Branch::remote(
        "inbox",
        folder.join("cache"),
        remote.to_str().expect("a UTF-8 path"),
    )
}

/// Each kind of branch over the bare repository `repo`: through a cache of
/// its own, and as a branch with no remote.
fn both(folder: &Path, repo: &Path) -> [Branch; 2] {
    [remote_branch(folder, repo), Branch::local("inbox", repo)]
}

#[test]
fn a_tip_read_stays_readable() {
    let folder = tempfile::tempdir().unwrap();
    let repo = bare(folder.path(), "repo.git");
    for branch in both(folder.path(), &repo) {
        let a = seed(
            &repo,
            "inbox",
            &[
                ("inbox/moves/moves.rec", "format: weir-move/1\n"),
                ("README", "an inbox\n"),
            ],
            "start",
            "operator@one",
        );
        let at_a = read(&branch).unwrap().expect("a tip");
        assert_eq!(at_a.commit(), a);

        let b = seed(
            &repo,
            "inbox",
            &[
                (
                    "inbox/moves/moves.rec",
                    "format: weir-move/1\n\nformat: weir-move/1\n",
                ),
                ("inbox/items/x/item.rec", "format: weir-item/1\n"),
            ],
            "moved",
            "curation@two",
        );

        assert_eq!(
            at_a.file("inbox/moves/moves.rec").unwrap().as_deref(),
            Some("format: weir-move/1\n")
        );
        assert_eq!(at_a.file("inbox/items/x/item.rec").unwrap(), None);
        let under_inbox = at_a.files("inbox/").unwrap();
        assert_eq!(
            under_inbox.keys().collect::<Vec<_>>(),
            ["inbox/moves/moves.rec"]
        );
        assert_eq!(at_a.files("").unwrap().len(), 2);

        assert_eq!(read(&branch).unwrap().expect("a tip").commit(), b);
        assert_eq!(tip(&repo, "inbox"), Some(b));
        // The next kind of branch starts from none.
        common::git(&repo, &["update-ref", "-d", "refs/heads/inbox"]);
    }
}

#[test]
fn a_branch_that_does_not_exist() {
    let folder = tempfile::tempdir().unwrap();
    let repo = bare(folder.path(), "repo.git");
    seed(&repo, "other", &[("a", "a\n")], "start", "operator@one");
    let before = refs(&repo);
    for branch in both(folder.path(), &repo) {
        assert_eq!(read(&branch).unwrap(), None, "{branch:?}");
        assert_eq!(refs(&repo), before);
    }
    // The cache was made, and holds no ref for the branch.
    assert_eq!(refs(&folder.path().join("cache")), "");
}

#[test]
fn a_remote_read_fetches_into_the_cache_made_when_missing() {
    let folder = tempfile::tempdir().unwrap();
    let remote = bare(folder.path(), "remote.git");
    let a = seed(&remote, "inbox", &[("a", "a\n")], "start", "operator@one");
    let branch = remote_branch(folder.path(), &remote);
    assert!(!folder.path().join("cache").exists());

    read(&branch).unwrap();

    let cache = folder.path().join("cache");
    assert!(cache.join("HEAD").is_file(), "no cache made");
    assert_eq!(refs(&cache), format!("{a} refs/weir/inbox"));
    assert_eq!(branch.git_dir(), cache);
}

#[test]
fn a_file_that_is_not_utf8_is_named() {
    let folder = tempfile::tempdir().unwrap();
    let repo = bare(folder.path(), "repo.git");
    land_raw(
        &repo,
        "inbox",
        &[
            (b"inbox/latin1.txt", b"caf\xe9\n"),
            (b"inbox/ok.txt", b"ok\n"),
        ],
        "start",
        "operator@one",
    );
    for branch in both(folder.path(), &repo) {
        let at = read(&branch).unwrap().unwrap();
        let error = at.file("inbox/latin1.txt").unwrap_err();
        assert_eq!(error.path, "inbox/latin1.txt");
        assert!(error.reason.contains("not UTF-8"), "{error}");
        let error = at.files("inbox/").unwrap_err();
        assert_eq!(error.path, "inbox/latin1.txt");
        assert_eq!(at.file("inbox/ok.txt").unwrap().as_deref(), Some("ok\n"));
    }
}

#[test]
fn a_path_that_is_not_utf8_is_named() {
    let folder = tempfile::tempdir().unwrap();
    let repo = bare(folder.path(), "repo.git");
    land_raw(
        &repo,
        "inbox",
        &[(b"inbox/caf\xe9.txt", b"text\n")],
        "start",
        "operator@one",
    );
    let at = read(&Branch::local("inbox", &repo)).unwrap().unwrap();
    let error = at.files("").unwrap_err();
    assert!(error.path.starts_with("inbox/caf"), "{error}");
    assert!(error.reason.contains("not UTF-8"), "{error}");
}

#[test]
fn a_folder_is_no_file() {
    let folder = tempfile::tempdir().unwrap();
    let repo = bare(folder.path(), "repo.git");
    seed(
        &repo,
        "inbox",
        &[("inbox/a/b", "b\n")],
        "start",
        "operator@one",
    );
    let at = read(&Branch::local("inbox", &repo)).unwrap().unwrap();
    let error = at.file("inbox/a").unwrap_err();
    assert!(error.reason.contains("folder"), "{error}");
    assert!(at.file("../x").is_err());
}

#[test]
fn a_repository_that_is_not_one_fails_the_read() {
    let folder = tempfile::tempdir().unwrap();
    let failure = read(&Branch::local("inbox", folder.path().join("none"))).unwrap_err();
    assert_eq!(failure.step, Step::Read);
    let failure = read(&remote_branch(
        folder.path(),
        &folder.path().join("none.git"),
    ))
    .unwrap_err();
    assert_eq!(failure.step, Step::Read);
}
