//! Asking for a write, forming it at the tip, checking it and committing
//! it, held to the scenarios of shared-branch-write (44 to 47, 50, 71).

mod common;

use std::cell::Cell;

use common::{ask, now, seed, write_of, Stage};
use weir_branch::{write, Branch, WriteError};

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
