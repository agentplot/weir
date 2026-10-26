//! Reading an inbox whole from one tip, held to the scenarios of
//! inbox-layout: every record where the layout places it, in a format and
//! major this crate reads (40, 42, 43).

mod common;

use weir_signals::{InboxAt, InboxError};

use common::capture;

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
