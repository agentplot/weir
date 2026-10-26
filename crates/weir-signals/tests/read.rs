//! Reading an inbox whole from one tip, held to the scenarios of
//! inbox-layout: every record where the layout places it, in a format and
//! major this crate reads (40, 42, 43).

mod common;

use weir_signals::{InboxAt, InboxError};

use common::{capture, key, read};

fn refused(files: &[(&str, &str)]) -> InboxError {
    InboxAt::read(files.iter().copied()).expect_err("the inbox is refused")
}

const MOVE: &str = "format: weir-move/1\n\
                    signal: release/a#1\n\
                    move: drop\n\
                    target: noise\n\
                    by: curation\n\
                    at: 2026-10-25T09:00:00Z\n";

#[test]
fn a_later_major() {
    let later = MOVE.replace("weir-move/1", "weir-move/2");
    let error = refused(&[("inbox/moves/moves.rec", &later)]);
    assert_eq!(error.path, "inbox/moves/moves.rec");
    assert_eq!(error.record, Some(1));
    assert_eq!(error.field.as_deref(), Some("format"));
    assert_eq!(
        error.to_string(),
        "`inbox/moves/moves.rec`, record 1: `format` is `weir-move/2`, a later major than \
         this reader reads, which is 1"
    );
    // Every record of every place is read so: a later capture refuses too.
    let capture = capture("release/a")
        .to_rec()
        .replace("weir-capture/1", "weir-capture/2");
    let error = refused(&[("inbox/captures/release/a/capture.rec", &capture)]);
    assert!(error.reason.contains("a later major"), "{error}");
}

#[test]
fn a_record_of_another_format_where_it_lies() {
    let error = refused(&[("inbox/items/shim-timeout/sends.rec", MOVE)]);
    assert_eq!(error.path, "inbox/items/shim-timeout/sends.rec");
    assert_eq!(error.field.as_deref(), Some("format"));
    assert!(
        error
            .reason
            .contains("is `weir-move/1`, where `weir-send/1` lies"),
        "{error}"
    );
}

#[test]
fn a_capture_out_of_its_place() {
    let capture = capture("release/b").to_rec();
    let error = refused(&[("inbox/captures/release/a/capture.rec", &capture)]);
    assert_eq!(error.path, "inbox/captures/release/a/capture.rec");
    assert_eq!(error.field.as_deref(), Some("key"));
    assert_eq!(
        error.to_string(),
        "`inbox/captures/release/a/capture.rec`, record 1: `key` is `release/b`, and the \
         capture lies in the folder of `release/a`"
    );
}

#[test]
fn a_capture_rec_holds_one_capture() {
    let one = capture("release/a").to_rec();
    let two = format!("{one}\n{one}");
    let error = refused(&[("inbox/captures/release/a/capture.rec", &two)]);
    assert_eq!(error.path, "inbox/captures/release/a/capture.rec");
    assert!(error.reason.contains("holds 2 records"), "{error}");
    let error = refused(&[("inbox/captures/release/a/capture.rec", "")]);
    assert!(error.reason.contains("holds 0 records"), "{error}");
}

#[test]
fn signals_with_no_capture() {
    let error = refused(&[(
        "inbox/captures/release/a/signals.rec",
        "# weir-signal/1: read, no signals\n",
    )]);
    assert_eq!(error.path, "inbox/captures/release/a/signals.rec");
    assert!(error.reason.contains("beside no `capture.rec`"), "{error}");
}

#[test]
fn a_stray_file() {
    let error = refused(&[("inbox/moves/notes.txt", "a note")]);
    assert_eq!(error.path, "inbox/moves/notes.txt");
    assert!(
        error.reason.contains("the layout places no record"),
        "{error}"
    );
    // A folder that is no key is no capture's place.
    let error = refused(&[("inbox/captures/moves/x/capture.rec", "")]);
    assert_eq!(error.path, "inbox/captures/moves/x/capture.rec");
    assert!(error.reason.contains("which is no key"), "{error}");
}

#[test]
fn a_file_outside_the_inbox() {
    let capture = capture("release/a").to_rec();
    let inbox = read(&[
        ("README.md", "# not of the inbox\n"),
        ("inboxes/notes.txt", "nor this"),
        ("inbox/captures/release/a/capture.rec", &capture),
    ]);
    assert!(inbox.holds(&key("release/a")));
    assert_eq!(
        inbox.files().keys().collect::<Vec<_>>(),
        ["inbox/captures/release/a/capture.rec"]
    );
}

#[test]
fn an_empty_tip() {
    let inbox = read(&[("README.md", "# a repository with no inbox yet\n")]);
    assert_eq!(inbox.keys().count(), 0);
    assert!(inbox.files().is_empty());
    let inbox = InboxAt::read(Vec::<(String, String)>::new()).unwrap();
    assert!(!inbox.holds(&key("release/a")));
}

#[test]
fn a_record_with_no_format() {
    let no_format = format!("{MOVE}\n{}", MOVE.replace("format: weir-move/1\n", ""));
    let error = refused(&[("inbox/moves/moves.rec", &no_format)]);
    assert_eq!(error.path, "inbox/moves/moves.rec");
    assert_eq!(error.record, Some(2));
    assert_eq!(error.field.as_deref(), Some("format"));
    assert_eq!(
        error.to_string(),
        "`inbox/moves/moves.rec`, record 2: `format` is missing"
    );
    let item = "format: weir-item/1\nitem: shim-timeout\nmade_by: join\n\
                named_by: curation\nnamed_at: 2026-10-25T09:00:00Z\n";
    let error = refused(&[
        ("inbox/items/shim-timeout/item.rec", item),
        ("inbox/items/shim-timeout/brief.md", "# Shim\n"),
    ]);
    assert_eq!(error.path, "inbox/items/shim-timeout/brief.md");
    assert!(error.reason.contains("names its format"), "{error}");
}

#[test]
fn a_path_given_twice() {
    let capture = capture("release/a").to_rec();
    let error = refused(&[
        ("inbox/captures/release/a/capture.rec", &capture),
        ("inbox/captures/release/a/capture.rec", &capture),
    ]);
    assert!(error.reason.contains("given twice"), "{error}");
}

#[test]
fn reading_changes_nothing_it_was_handed() {
    let capture = capture("release/a").to_rec();
    let files = [(
        "inbox/captures/release/a/capture.rec".to_string(),
        capture.clone(),
    )];
    let inbox = InboxAt::read(files.iter().map(|(path, text)| (path, text))).unwrap();
    assert_eq!(files[0].1, capture);
    assert_eq!(
        inbox.files()["inbox/captures/release/a/capture.rec"],
        capture
    );
}
