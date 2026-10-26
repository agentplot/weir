//! Landing a write only while the branch stands at the tip read, and
//! forming it again on a moved branch up to the writer's bound, held to
//! the scenarios of shared-branch-write (44 to 46, 48, 79, 80; S5, S6, S7).

mod common;

use std::cell::Cell;
use std::fs;
use std::path::Path;

use common::{
    all_commits, ask, commits_since, files, message, now, parents, refs, seed, tip, write_of, Stage,
};
use weir_branch::{write, Branch, Files, Landed, Nothing, Tip, WriteError, Written};

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

// Formed again on a moved branch

/// Adds `path` unless the tip holds it: a held key (14).
fn add(tip: Option<&Tip>, path: &str, text: &str) -> Result<Files, String> {
    let held = match tip {
        Some(tip) => tip.file(path).map_err(|error| error.to_string())?,
        None => None,
    };
    Ok(match held {
        Some(_) => Files::new(),
        None => write_of(&[(path, text)]),
    })
}

/// Appends `text` to `path`, refused when the tip's text holds the line
/// `refused_when`: a standing move (29).
fn append(tip: Option<&Tip>, path: &str, text: &str, refused_when: &str) -> Result<Files, String> {
    let before = match tip {
        Some(tip) => tip
            .file(path)
            .map_err(|error| error.to_string())?
            .unwrap_or_default(),
        None => String::new(),
    };
    if before.lines().any(|line| line == refused_when) {
        return Err(refused_when.to_string());
    }
    let joined = if before.is_empty() {
        text.to_string()
    } else {
        format!("{before}\n{text}")
    };
    Ok(write_of(&[(path, &joined)]))
}

#[test]
fn two_writes_from_one_tip_both_land_one_after_the_other() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let first = Cell::new(None);
    let tried = Cell::new(0);
    let second = landed(
        write(
            &stage.branch("two"),
            &ask("add c", "curation@two", &now),
            |tip| {
                tried.set(tried.get() + 1);
                if tried.get() == 1 {
                    assert_eq!(tip.unwrap().commit(), a);
                    let landed = landed(
                        write(
                            &stage.branch("one"),
                            &ask("add b", "operator@one", &now),
                            |_| Ok(write_of(&[("inbox/b", "b\n")])),
                        )
                        .unwrap(),
                    );
                    assert_eq!(landed.on.as_deref(), Some(a.as_str()));
                    first.set(Some(landed.commit));
                }
                Ok(write_of(&[("inbox/c", "c\n")]))
            },
        )
        .unwrap(),
    );
    let first = first.take().unwrap();
    assert_eq!(second.tries, 2);
    assert_eq!(second.on.as_deref(), Some(first.as_str()));
    assert_eq!(parents(&stage.remote, &second.commit), vec![first.clone()]);
    assert_eq!(
        commits_since(&stage.remote, Some(&a), &second.commit),
        vec![first, second.commit.clone()]
    );
    assert_eq!(files(&stage.remote, &second.commit).len(), 3);
}

#[test]
fn two_machines_take_one_release_at_once() {
    let stage = Stage::new();
    seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let path = "inbox/captures/release/acme/v1/capture.rec";
    let capture = "format: weir-capture/1\nkey: release/acme/v1\n";
    let tried = Cell::new(0);
    let written = write(
        &stage.branch("two"),
        &ask("take release/acme/v1", "host@two", &now),
        |tip| {
            tried.set(tried.get() + 1);
            if tried.get() == 1 {
                write(
                    &stage.branch("one"),
                    &ask("take release/acme/v1", "host@one", &now),
                    |tip| add(tip, path, capture),
                )
                .unwrap();
            }
            add(tip, path, capture)
        },
    )
    .unwrap();
    let at = stage.tip();
    assert_eq!(written, Written::Nothing(Nothing { at, tries: 2 }));
    let at = stage.tip().unwrap();
    assert_eq!(
        message(&stage.remote, &at),
        "take release/acme/v1\n\nAsked-by: host@one\n"
    );
    assert_eq!(files(&stage.remote, &at)[path], capture);
}

#[test]
fn two_moves_of_one_signal_in_one_second() {
    let stage = Stage::new();
    seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let path = "inbox/moves/moves.rec";
    let signal = "signal: release/acme/v1#1";
    let curation = format!("format: weir-move/1\n{signal}\nword: join\nby: curation@one\n");
    let operator = format!("format: weir-move/1\n{signal}\nword: drop\nby: operator@two\n");
    let tried = Cell::new(0);
    let refused = write(
        &stage.branch("two"),
        &ask("move release/acme/v1#1", "operator@two", &now),
        |tip| {
            tried.set(tried.get() + 1);
            if tried.get() == 1 {
                write(
                    &stage.branch("one"),
                    &ask("move release/acme/v1#1", "curation@one", &now),
                    |tip| append(tip, path, &curation, signal),
                )
                .unwrap();
            }
            append(tip, path, &operator, signal)
        },
    )
    .unwrap_err();
    let at = stage.tip();
    assert_eq!(
        refused,
        WriteError::Refused {
            at: at.clone(),
            reason: signal.to_string(),
            tries: 2,
        }
    );
    let moves = &files(&stage.remote, &at.unwrap())[path];
    assert_eq!(moves.matches(signal).count(), 1);
    assert!(moves.contains("curation@one"));
}

#[test]
fn the_first_write_meets_a_branch_made_meanwhile() {
    let folder = tempfile::tempdir().unwrap();
    let repo = common::bare(folder.path(), "repo.git");
    for branch in both(folder.path(), &repo) {
        let made = Cell::new(None);
        let landed = landed(
            write(&branch, &ask("add b", "operator@one", &now), |tip| {
                if tip.is_none() {
                    made.set(Some(seed(
                        &repo,
                        "inbox",
                        &[("inbox/a", "a\n")],
                        "first",
                        "curation@two",
                    )));
                }
                Ok(write_of(&[("inbox/b", "b\n")]))
            })
            .unwrap(),
        );
        let made = made.take().unwrap();
        assert_eq!(landed.tries, 2);
        assert_eq!(parents(&repo, &landed.commit), vec![made]);
        common::git(&repo, &["update-ref", "-d", "refs/heads/inbox"]);
    }
}

/// Writes `b` with `bound`, another write landing between every read and
/// its move; gives the outcome and how many times the write was formed.
fn kept_moving(bound: Option<u32>) -> (WriteError, u32) {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let tried = Cell::new(0);
    let mut ask = ask("add b", "operator@one", &now);
    ask.bound = bound;
    let refused = write(&stage.branch("cache"), &ask, |_| {
        tried.set(tried.get() + 1);
        let n = tried.get();
        seed(
            &stage.remote,
            "inbox",
            &[(&format!("inbox/meanwhile/{n}"), "x\n")],
            &format!("meanwhile {n}"),
            "curation@two",
        );
        Ok(write_of(&[("inbox/b", "b\n")]))
    })
    .unwrap_err();
    let tip = stage.tip().unwrap();
    assert!(
        !files(&stage.remote, &tip).contains_key("inbox/b"),
        "the write landed"
    );
    assert_eq!(
        commits_since(&stage.remote, Some(&a), &tip).len() as u32,
        tried.get()
    );
    (refused, tried.get())
}

#[test]
fn a_branch_that_keeps_moving() {
    let (refused, tried) = kept_moving(Some(2));
    assert_eq!(refused, WriteError::KeptMoving { tries: 3, bound: 2 });
    assert_eq!(tried, 3);
    assert!(refused.to_string().contains("tried 3 times"), "{refused}");
}

#[test]
fn the_bound_left_out() {
    let (refused, tried) = kept_moving(None);
    assert_eq!(refused, WriteError::KeptMoving { tries: 6, bound: 5 });
    assert_eq!(tried, 6);
}

#[test]
fn a_remote_that_refuses_every_push() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let hook = stage.remote.join("hooks/pre-receive");
    fs::write(
        &hook,
        "#!/bin/sh\necho 'this remote takes no push' >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let tried = Cell::new(0);
    let refused = write(
        &stage.branch("cache"),
        &ask("add b", "operator@one", &now),
        |_| {
            tried.set(tried.get() + 1);
            Ok(write_of(&[("inbox/b", "b\n")]))
        },
    )
    .unwrap_err();
    match &refused {
        WriteError::Failed { step, said } => {
            assert_eq!(*step, weir_branch::Step::Move);
            assert!(said.contains("this remote takes no push"), "{said}");
        }
        other => panic!("not failed: {other:?}"),
    }
    assert_eq!(tried.get(), 1);
    assert_eq!(stage.tip(), Some(a.clone()));
    assert_eq!(all_commits(&stage.remote), vec![a]);
}
