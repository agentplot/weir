//! Landing a write only while the branch stands at the tip read, and
//! forming it again on a moved branch up to the writer's bound, held to
//! the scenarios of shared-branch-write (44 to 46, 48, 79, 80; S5, S6, S7).

mod common;

use std::path::Path;

use common::{ask, now, parents, refs, seed, tip, write_of};
use weir_branch::{write, Branch, Landed, Written};

fn landed(written: Written) -> Landed {
    match written {
        Written::Landed(landed) => landed,
        other => panic!("not landed: {other:?}"),
    }
}

/// Each kind of branch over the bare repository `repo`: through the cache
/// `cache` beside it, and as a branch with no remote.
fn both(folder: &Path, repo: &Path) -> [Branch; 2] {
    [
        Branch::remote("inbox", folder.join("cache"), repo.to_str().unwrap()),
        Branch::local("inbox", repo),
    ]
}

// Landing on the tip read

#[test]
fn the_branch_stands_where_it_was_read() {
    let folder = tempfile::tempdir().unwrap();
    let repo = common::bare(folder.path(), "repo.git");
    for branch in both(folder.path(), &repo) {
        let a = seed(
            &repo,
            "inbox",
            &[("inbox/a", "a\n")],
            "start",
            "operator@one",
        );
        let landed = landed(
            write(&branch, &ask("add b", "operator@one", &now), |_| {
                Ok(write_of(&[("inbox/b", "b\n")]))
            })
            .unwrap(),
        );
        assert_eq!(landed.tries, 1);
        assert_eq!(landed.on.as_deref(), Some(a.as_str()));
        assert_eq!(parents(&repo, &landed.commit), vec![a]);
        assert_eq!(tip(&repo, "inbox"), Some(landed.commit.clone()));
        if let weir_branch::Place::Remote { cache, .. } = branch.place() {
            assert_eq!(
                refs(cache),
                format!("{} refs/weir/inbox", landed.commit),
                "the cache's ref does not follow the push"
            );
        }
        common::git(&repo, &["update-ref", "-d", "refs/heads/inbox"]);
    }
}

#[test]
fn the_first_write_makes_the_branch() {
    let folder = tempfile::tempdir().unwrap();
    let repo = common::bare(folder.path(), "repo.git");
    for branch in both(folder.path(), &repo) {
        assert_eq!(tip(&repo, "inbox"), None);
        let landed = landed(
            write(&branch, &ask("add a", "operator@one", &now), |tip| {
                assert!(tip.is_none(), "a tip on a branch that does not exist");
                Ok(write_of(&[("inbox/a", "a\n")]))
            })
            .unwrap(),
        );
        assert_eq!(landed.on, None);
        assert!(parents(&repo, &landed.commit).is_empty());
        assert_eq!(tip(&repo, "inbox"), Some(landed.commit));
        common::git(&repo, &["update-ref", "-d", "refs/heads/inbox"]);
    }
}
