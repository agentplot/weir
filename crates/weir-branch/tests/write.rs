//! Asking for a write, forming it at the tip, checking it and committing
//! it, held to the scenarios of shared-branch-write (44 to 47, 50, 71).

mod common;

use std::cell::Cell;

use common::{all_commits, ask, now, seed, write_of, Stage};
use weir_branch::{write, Branch, Files, Landed, Nothing, WriteError, Written};

const MOVE: &str = "format: weir-move/1\nsignal: release/acme/v1#1\nword: drop\n";

fn landed(written: Written) -> Landed {
    match written {
        Written::Landed(landed) => landed,
        other => panic!("not landed: {other:?}"),
    }
}

// The ask

#[test]
fn no_one_named() {
    let stage = Stage::new();
    let start = seed(
        &stage.remote,
        "inbox",
        &[("a", "a\n")],
        "start",
        "operator@one",
    );
    let formed = Cell::new(0);
    for (what, who, field) in [
        ("take release/acme/v1", "", "who"),
        ("take release/acme/v1", "  ", "who"),
        ("take release/acme/v1", "operator\n@one", "who"),
        ("", "operator@one", "what"),
        ("take\nrelease/acme/v1", "operator@one", "what"),
    ] {
        let refused = write(&stage.branch("cache"), &ask(what, who, &now), |_| {
            formed.set(formed.get() + 1);
            Ok(write_of(&[("b", "b\n")]))
        })
        .unwrap_err();
        match &refused {
            WriteError::Asked { field: named, .. } => assert_eq!(*named, field, "{refused}"),
            other => panic!("not refused as asked: {other:?}"),
        }
        assert!(refused.to_string().contains(field), "{refused}");
    }
    assert_eq!(formed.get(), 0, "a write was formed");
    assert!(!stage.cache("cache").exists(), "the cache was made");
    assert_eq!(stage.tip(), Some(start));
}

#[test]
fn a_password_in_the_address() {
    let folder = tempfile::tempdir().unwrap();
    let cache = folder.path().join("cache");
    let branch = Branch::remote("inbox", &cache, "https://bot:s3cret@example.org/inbox.git");
    let refused = write(
        &branch,
        &ask("take release/acme/v1", "operator@one", &now),
        |_| panic!("a write was formed"),
    )
    .unwrap_err();
    match &refused {
        WriteError::Asked { field, reason } => {
            assert_eq!(*field, "url");
            assert!(
                reason.contains("https://bot@example.org/inbox.git"),
                "{reason}"
            );
        }
        other => panic!("not refused as asked: {other:?}"),
    }
    assert!(!refused.to_string().contains("s3cret"), "{refused}");
    assert!(!format!("{refused:?}").contains("s3cret"), "{refused:?}");
    assert!(!cache.exists(), "git ran: the cache was made");
}

#[test]
fn a_branch_git_cannot_be_handed() {
    let stage = Stage::new();
    let branch = Branch::remote(
        "-inbox",
        stage.cache("cache"),
        stage.remote.to_str().unwrap(),
    );
    let refused = write(&branch, &ask("x", "operator@one", &now), |_| {
        panic!("a write was formed")
    })
    .unwrap_err();
    assert!(
        matches!(
            refused,
            WriteError::Asked {
                field: "branch",
                ..
            }
        ),
        "{refused:?}"
    );
}

// Forming and checking at the tip

#[test]
fn a_check_refuses_the_write_at_the_tip() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/moves/moves.rec", MOVE)],
        "start",
        "operator@one",
    );
    let branch = stage.branch("cache");
    let refused = write(
        &branch,
        &ask("move release/acme/v1#1", "curation@two", &now),
        |tip| {
            let tip = tip.expect("a tip");
            let moves = tip
                .file("inbox/moves/moves.rec")
                .unwrap()
                .unwrap_or_default();
            if moves.contains("signal: release/acme/v1#1") {
                return Err("release/acme/v1#1 has a standing move".to_string());
            }
            Ok(write_of(&[("inbox/moves/moves.rec", "")]))
        },
    )
    .unwrap_err();
    assert_eq!(
        refused,
        WriteError::Refused {
            at: Some(a.clone()),
            reason: "release/acme/v1#1 has a standing move".to_string(),
            tries: 1,
        }
    );
    assert!(refused.to_string().contains(&a), "{refused}");
    assert_eq!(stage.tip(), Some(a.clone()));
    assert_eq!(all_commits(&stage.cache("cache")), vec![a.clone()]);
    assert_eq!(all_commits(&stage.remote), vec![a]);
}

#[test]
fn nothing_to_write() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[(
            "inbox/captures/release/acme/v1/capture.rec",
            "format: weir-capture/1\n",
        )],
        "start",
        "operator@one",
    );
    let written = write(
        &stage.branch("cache"),
        &ask("take release/acme/v1", "operator@two", &now),
        |tip| {
            let path = "inbox/captures/release/acme/v1/capture.rec";
            if tip.expect("a tip").file(path).unwrap().is_some() {
                return Ok(Files::new());
            }
            Ok(write_of(&[(path, "format: weir-capture/1\n")]))
        },
    )
    .unwrap();
    assert_eq!(
        written,
        Written::Nothing(Nothing {
            at: Some(a.clone()),
            tries: 1,
        })
    );
    assert_eq!(stage.tip(), Some(a.clone()));
    assert_eq!(all_commits(&stage.cache("cache")), vec![a]);
}

#[test]
fn a_record_recfix_refuses() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("a", "a\n")],
        "start",
        "operator@one",
    );
    let refused = write(
        &stage.branch("cache"),
        &ask("move release/acme/v1#1", "curation@two", &now),
        |_| {
            Ok(write_of(&[(
                "inbox/moves/moves.rec",
                "format: weir-move/1\nthis line is no field\n",
            )]))
        },
    )
    .unwrap_err();
    match &refused {
        WriteError::Recfix {
            at,
            path,
            said,
            tries,
        } => {
            assert_eq!(at.as_deref(), Some(a.as_str()));
            assert_eq!(path, "inbox/moves/moves.rec");
            assert!(said.starts_with("inbox/moves/moves.rec:"), "{said}");
            assert!(said.contains("error"), "{said}");
            assert!(!said.contains("stdin"), "{said}");
            assert_eq!(*tries, 1);
        }
        other => panic!("not refused by recfix: {other:?}"),
    }
    assert!(
        refused.to_string().contains("inbox/moves/moves.rec"),
        "{refused}"
    );
    assert_eq!(stage.tip(), Some(a.clone()));
    assert_eq!(all_commits(&stage.cache("cache")), vec![a]);
}

#[test]
fn only_record_files_are_held_to_recfix() {
    let stage = Stage::new();
    let landed = landed(
        write(
            &stage.branch("cache"),
            &ask("write a brief", "operator@one", &now),
            |_| {
                Ok(write_of(&[(
                    "inbox/items/x/brief.md",
                    "this line is no field\n",
                )]))
            },
        )
        .unwrap(),
    );
    assert_eq!(stage.tip(), Some(landed.commit));
}

#[test]
fn a_path_no_write_may_add() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/moves/moves.rec", MOVE), ("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    for (path, held) in [
        ("../outside", "`..`"),
        ("inbox//x", "empty segment"),
        ("/inbox/x", "not relative"),
        (".git/config", "`.git`"),
        ("inbox/moves", "folder"),
        ("inbox/a/b", "under the file `inbox/a`"),
    ] {
        let refused = write(
            &stage.branch("cache"),
            &ask("x", "operator@one", &now),
            |_| Ok(write_of(&[(path, "x\n")])),
        )
        .unwrap_err();
        match &refused {
            WriteError::Path {
                path: named,
                reason,
                ..
            } => {
                assert_eq!(named, path);
                assert!(reason.contains(held), "{path}: {reason}");
            }
            other => panic!("{path} not refused as a path: {other:?}"),
        }
    }
    assert_eq!(stage.tip(), Some(a));
}
