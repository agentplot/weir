//! The shared helpers do what the other tests rest on: a seeded branch
//! reads back as it was seeded, and `recfix` refuses what is no record.

mod common;

use std::collections::BTreeMap;

use common::{bare, commits_since, files, message, parents, recfix, recfix_says, seed, tip};

#[test]
fn a_seeded_branch_reads_back_as_it_was_seeded() {
    let folder = tempfile::tempdir().unwrap();
    let remote = bare(folder.path(), "remote.git");
    assert_eq!(tip(&remote, "inbox"), None);

    let first = seed(
        &remote,
        "inbox",
        &[("inbox/a.rec", "format: weir-move/1\n")],
        "the first",
        "operator@one",
    );
    assert!(parents(&remote, &first).is_empty());
    let second = seed(
        &remote,
        "inbox",
        &[("inbox/b.rec", "format: weir-move/1\n")],
        "the second",
        "curation@two",
    );

    assert_eq!(tip(&remote, "inbox"), Some(second.clone()));
    assert_eq!(parents(&remote, &second), vec![first.clone()]);
    assert_eq!(
        message(&remote, &second),
        "the second\n\nAsked-by: curation@two\n"
    );
    assert_eq!(
        commits_since(&remote, None, &second),
        vec![first.clone(), second.clone()]
    );
    assert_eq!(
        commits_since(&remote, Some(&first), &second),
        vec![second.clone()]
    );
    let expected: BTreeMap<String, String> = [
        (
            "inbox/a.rec".to_string(),
            "format: weir-move/1\n".to_string(),
        ),
        (
            "inbox/b.rec".to_string(),
            "format: weir-move/1\n".to_string(),
        ),
    ]
    .into();
    assert_eq!(files(&remote, &second), expected);
}

#[test]
fn recfix_takes_a_record_and_refuses_a_line_that_is_no_field() {
    recfix("format: weir-move/1\nsignal: release/acme/v1#1\n");
    let said = recfix_says("format: weir-move/1\nnot a field\n").expect("a refusal");
    assert!(said.contains("error"), "{said}");
}
