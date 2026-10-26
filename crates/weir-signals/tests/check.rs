//! The check every write passes, held to the scenarios of inbox-layout: a
//! write only adds, and is checked against the tip it is formed on, which
//! a write formed elsewhere meets too (41, 46; core.21, core.24, core.26).

mod common;

use weir_capture::rec;
use weir_signals::{InboxAt, Refused, Write};

use common::{a_move, after, capture, empty, moved, with_signals};

const AT: &str = "2026-10-25T09:00:00Z";

fn one(path: &str, text: &str) -> Write {
    let mut write = Write::new();
    write.put(path, text);
    write
}

fn refused_at(inbox: &InboxAt, write: &Write) -> Refused {
    inbox.check(write).expect_err("the write is refused")
}

/// An inbox with a capture read into two signals, a move, an item briefed
/// and sent, and a proposed batch.
fn full() -> InboxAt {
    let inbox = with_signals(&empty(), "release/x", 2);
    let inbox = moved(
        &inbox,
        &a_move("release/x#1", "join", "shim-timeout", "curation", AT),
    );
    let inbox = after(
        &inbox,
        &inbox.write_brief("shim-timeout", "the brief\n").unwrap(),
    );
    let inbox = after(
        &inbox,
        &inbox
            .send(&["shim-timeout"], "agentplot/weir fix", AT)
            .unwrap(),
    );
    after(
        &inbox,
        &inbox
            .propose_batch(
                "b",
                "curation",
                AT,
                &[a_move("release/x#2", "drop", "noise", "curation", AT)],
            )
            .unwrap(),
    )
}

#[test]
fn a_capture_rewritten() {
    let inbox = full();
    let path = "inbox/captures/release/x/capture.rec";
    let read = weir_capture::Capture {
        unread: false,
        ..capture("release/x")
    };
    let refused = refused_at(&inbox, &one(path, &read.to_rec()));
    assert_eq!(refused.0[0].what, path);
    assert!(refused.0[0].why.contains("never changes"), "{refused}");
    // Even as it stands.
    let same = inbox.files()[path].clone();
    assert!(inbox.check(&one(path, &same)).is_err());
}

#[test]
fn signals_and_items_are_never_rewritten() {
    let inbox = full();
    for path in [
        "inbox/captures/release/x/signals.rec",
        "inbox/items/shim-timeout/item.rec",
    ] {
        let text = inbox.files()[path].clone();
        let refused = refused_at(&inbox, &one(path, &text));
        assert_eq!(refused.0[0].what, path);
    }
}

#[test]
fn a_move_edited_in_place() {
    let inbox = full();
    let path = "inbox/moves/moves.rec";
    let tip = inbox.files()[path].clone();
    let appended = inbox
        .make_move(&a_move("release/x#2", "drop", "noise", "operator", AT))
        .unwrap()
        .files[path]
        .clone();
    let edited = appended.replacen("target: shim-timeout", "target: pin-mise", 1);
    assert!(!edited.starts_with(&tip));
    let refused = refused_at(&inbox, &one(path, &edited));
    assert_eq!(refused.0[0].what, path);
    assert!(refused.0[0].why.contains("at a record's end"), "{refused}");
}

#[test]
fn an_appended_file_keeps_every_record_it_held() {
    let inbox = full();
    for path in [
        "inbox/moves/moves.rec",
        "inbox/moves/batches/b.rec",
        "inbox/items/shim-timeout/sends.rec",
    ] {
        let tip = inbox.files()[path].clone();
        // A field added to the tip's last record, though the text begins
        // with the tip's.
        let grown = format!("{tip}extra: a field\n");
        let refused = refused_at(&inbox, &one(path, &grown));
        assert_eq!(refused.0[0].what, path, "{refused}");
        // A file emptied.
        assert!(inbox.check(&one(path, "")).is_err(), "{path}");
    }
}

#[test]
fn a_brief_rewritten() {
    let inbox = full();
    let write = inbox
        .write_brief("shim-timeout", "a later brief\n")
        .unwrap();
    let after = inbox.check(&write).unwrap();
    assert_eq!(
        after.item("shim-timeout").unwrap().brief.as_deref(),
        Some("a later brief\n")
    );
}

#[test]
fn formed_again_on_a_new_tip() {
    let tip = with_signals(&empty(), "release/x", 1);
    let drop = a_move("release/x#1", "drop", "noise", "curation", AT);
    let join = a_move("release/x#1", "join", "shim-timeout", "operator", AT);
    let join_write = tip.make_move(&join).unwrap();
    let next = after(&tip, &tip.make_move(&drop).unwrap());
    // Formed again on the new tip, the join meets the standing drop...
    let refused = next.make_move(&join).unwrap_err();
    assert!(refused.0[0].why.contains("a drop by curation"), "{refused}");
    // ...and its write formed on the old tip is refused there too.
    let refused = refused_at(&next, &join_write);
    assert_eq!(refused.0[0].what, "inbox/moves/moves.rec");
}

#[test]
fn a_write_formed_elsewhere() {
    let inbox = full();
    let path = "inbox/moves/moves.rec";
    let tip = inbox.files()[path].clone();
    let promote = "format: weir-move/1\nsignal: release/x#2\nmove: promote\n\
                   target: the roadmap\nby: curation\nat: 2026-10-25T09:00:00Z\n";
    let refused = refused_at(&inbox, &one(path, &format!("{tip}\n{promote}")));
    assert_eq!(refused.0[0].what, path);
    assert_eq!(refused.0[0].field.as_deref(), Some("move"));
    assert!(refused.0[0].why.contains("`promote`"), "{refused}");
    // One that keeps every rule is taken, and gives the inbox after it.
    let drop = promote.replace("move: promote", "move: drop");
    let after = inbox.check(&one(path, &format!("{tip}\n{drop}"))).unwrap();
    assert!(after.standing(&common::signal("release/x#2")).is_some());
}

#[test]
fn a_write_outside_the_inbox() {
    let refused = refused_at(&full(), &one("README.md", "# changed\n"));
    assert_eq!(refused.0[0].what, "README.md");
    let refused = refused_at(&full(), &one("inbox/moves/notes.txt", "a note"));
    assert_eq!(refused.0[0].what, "inbox/moves/notes.txt");
}

#[test]
fn the_inbox_after_one_write_takes_the_next() {
    let inbox = with_signals(&empty(), "release/x", 2);
    let first = inbox
        .make_move(&a_move("release/x#1", "drop", "noise", "curation", AT))
        .unwrap();
    let next = inbox.check(&first).unwrap();
    let second = next
        .make_move(&a_move("release/x#2", "drop", "noise", "curation", AT))
        .unwrap();
    let last = next.check(&second).unwrap();
    assert_eq!(last.moves().len(), 2);
    assert_eq!(
        rec::read(&last.files()["inbox/moves/moves.rec"])
            .unwrap()
            .len(),
        2
    );
}
