//! Moves, held to the scenarios of inbox-signals and inbox-layout: one
//! standing move per signal, replaced only by number, one of six words
//! with its consequence (29 to 31; S6).

mod common;

use weir_signals::{InboxAt, Item, MadeBy, Move, NotTaken, Outcome, SendRecord, Word, Write};

use common::{a_move, after, empty, moved, paths, recsel_value, signal, with_signals};

const CURATION_AT: &str = "2026-10-25T09:00:00Z";
const OPERATOR_AT: &str = "2026-10-25T09:00:00.4+02:00";

/// An inbox holding `release/x` read into three signals.
fn three() -> InboxAt {
    with_signals(&empty(), "release/x", 3)
}

fn refused(inbox: &InboxAt, new: &weir_signals::NewMove) -> NotTaken {
    let refused = inbox.make_move(new).expect_err("the move is refused");
    assert_eq!(refused.0.len(), 1, "{refused}");
    refused.0[0].clone()
}

#[test]
fn a_move_stands_once_written() {
    let inbox = three();
    let drop = a_move("release/x#1", "drop", "noise", "curation", CURATION_AT);
    let write = inbox.make_move(&drop).unwrap();
    assert_eq!(paths(&write), ["inbox/moves/moves.rec"]);
    let text = &write.files["inbox/moves/moves.rec"];
    assert_eq!(
        text,
        "format: weir-move/1\n\
         signal: release/x#1\n\
         move: drop\n\
         target: noise\n\
         by: curation\n\
         at: 2026-10-25T09:00:00Z\n"
    );
    let inbox = after(&inbox, &write);
    let (number, standing) = inbox.standing(&signal("release/x#1")).unwrap();
    assert_eq!(number, 1);
    assert_eq!(standing.word, Word::Drop);
    assert_eq!(standing.target, "noise");
    assert_eq!(inbox.standing(&signal("release/x#2")), None);
}

#[test]
fn two_moves_within_a_second() {
    // S6: both formed on one tip; the drop lands, and the join, formed
    // again on the new tip, meets it.
    let tip = three();
    let drop = a_move("release/x#2", "drop", "noise", "curation", CURATION_AT);
    let join = a_move(
        "release/x#2",
        "join",
        "shim-timeout",
        "operator",
        OPERATOR_AT,
    );
    let drop_write = tip.make_move(&drop).unwrap();
    assert!(tip.make_move(&join).is_ok());
    let next = after(&tip, &drop_write);
    let not = refused(&next, &join);
    assert_eq!(not.what, "release/x#2");
    assert_eq!(not.field, None);
    assert_eq!(
        not.why,
        "`release/x#2` has a standing move, move 1 of `release/x#2`, a drop by curation at \
         2026-10-25T09:00:00Z; only a replacement naming it changes it"
    );
}

#[test]
fn a_move_for_no_signal() {
    let inbox = three();
    let not = refused(
        &inbox,
        &a_move("release/x#4", "drop", "noise", "curation", CURATION_AT),
    );
    assert_eq!(not.what, "release/x#4");
    assert_eq!(not.field.as_deref(), Some("signal"));
    let not = refused(
        &inbox,
        &a_move("release/y#1", "drop", "noise", "curation", CURATION_AT),
    );
    assert_eq!(not.what, "release/y#1");
}

#[test]
fn a_drop_replaced_by_a_join() {
    let inbox = moved(
        &three(),
        &a_move("release/x#1", "drop", "noise", "curation", CURATION_AT),
    );
    let before = inbox.files()["inbox/moves/moves.rec"].clone();
    let join = weir_signals::NewMove {
        replaces: Some(1),
        ..a_move(
            "release/x#1",
            "join",
            "shim-timeout",
            "operator",
            OPERATOR_AT,
        )
    };
    let write = inbox.make_move(&join).unwrap();
    let text = &write.files["inbox/moves/moves.rec"];
    assert!(text.starts_with(&before), "{text}");
    let added = &text[before.len()..];
    assert_eq!(recsel_value(added, "replaces"), "1");
    assert_eq!(recsel_value(added, "by"), "operator");
    let inbox = after(&inbox, &write);
    assert_eq!(inbox.moves().len(), 2);
    assert_eq!(inbox.moves()[0].word, Word::Drop);
    let (number, standing) = inbox.standing(&signal("release/x#1")).unwrap();
    assert_eq!(number, 2);
    assert_eq!(standing.word, Word::Join);
    assert_eq!(standing.replaces, Some(1));
}

#[test]
fn replacing_a_move_that_no_longer_stands() {
    let inbox = moved(
        &three(),
        &a_move("release/x#1", "drop", "noise", "curation", CURATION_AT),
    );
    let inbox = moved(
        &inbox,
        &weir_signals::NewMove {
            replaces: Some(1),
            ..a_move(
                "release/x#1",
                "route",
                "an ask upstream",
                "operator",
                OPERATOR_AT,
            )
        },
    );
    let not = refused(
        &inbox,
        &weir_signals::NewMove {
            replaces: Some(1),
            ..a_move(
                "release/x#1",
                "join",
                "shim-timeout",
                "operator",
                OPERATOR_AT,
            )
        },
    );
    assert_eq!(not.field.as_deref(), Some("replaces"));
    assert!(
        not.why
            .contains("the move standing is move 2 of `release/x#1`, a route"),
        "{}",
        not.why
    );
    // A first move replaces nothing.
    let not = refused(
        &three(),
        &weir_signals::NewMove {
            replaces: Some(1),
            ..a_move("release/x#1", "drop", "noise", "curation", CURATION_AT)
        },
    );
    assert_eq!(not.field.as_deref(), Some("replaces"));
}

#[test]
fn a_seventh_word() {
    let not = refused(
        &three(),
        &a_move(
            "release/x#1",
            "promote",
            "the roadmap",
            "curation",
            CURATION_AT,
        ),
    );
    assert_eq!(not.field.as_deref(), Some("move"));
    assert_eq!(
        not.why,
        "is `promote`, not one of drop, answered, challenge, join, attach, route"
    );
}

#[test]
fn a_drop_without_a_reason() {
    let not = refused(
        &three(),
        &a_move("release/x#1", "drop", "", "curation", CURATION_AT),
    );
    assert_eq!(not.field.as_deref(), Some("target"));
    assert_eq!(not.why, "is missing, and a drop carries its reason");
    for (word, consequence) in [
        ("answered", "the claim or record that settled the signal"),
        ("challenge", "the claim it weighs against"),
        ("join", "the item it makes or grows"),
        ("attach", "the sent item it adds to"),
        ("route", "the chore or the ask it offers"),
    ] {
        let not = refused(
            &three(),
            &a_move("release/x#1", word, "", "curation", CURATION_AT),
        );
        assert!(not.why.ends_with(consequence), "{word}: {}", not.why);
    }
}

#[test]
fn a_move_names_who_made_it_and_when() {
    let not = refused(
        &three(),
        &a_move("release/x#1", "drop", "noise", "", CURATION_AT),
    );
    assert_eq!(not.field.as_deref(), Some("by"));
    let not = refused(
        &three(),
        &a_move(
            "release/x#1",
            "drop",
            "noise",
            "curation",
            "2026-10-25 09:00",
        ),
    );
    assert_eq!(not.field.as_deref(), Some("at"));
}

#[test]
fn a_join_that_makes_an_item() {
    let inbox = three();
    let join = a_move(
        "release/x#1",
        "join",
        "shim-timeout",
        "curation",
        CURATION_AT,
    );
    let write = inbox.make_move(&join).unwrap();
    // The move and the item where the layout puts them.
    assert_eq!(
        paths(&write),
        ["inbox/items/shim-timeout/item.rec", "inbox/moves/moves.rec"]
    );
    let inbox = after(&inbox, &write);
    let item = inbox.item("shim-timeout").unwrap();
    assert_eq!(
        item.item,
        Item {
            item: "shim-timeout".into(),
            made_by: MadeBy::Join,
            named_by: "curation".into(),
            named_at: CURATION_AT.into(),
        }
    );
    assert!(!item.is_sent());
    assert_eq!(item.brief, None);
    assert_eq!(
        inbox.standing(&signal("release/x#1")).unwrap().1.target,
        "shim-timeout"
    );
    // A second join grows it, and writes no item again.
    let write = inbox
        .make_move(&a_move(
            "release/x#2",
            "join",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ))
        .unwrap();
    assert_eq!(paths(&write), ["inbox/moves/moves.rec"]);
}

/// `inbox` with the item `name` given a brief and sent, through a write
/// formed elsewhere, as a caller might hand it to the check.
fn sent(inbox: &InboxAt, name: &str) -> InboxAt {
    let mut write = Write::new();
    write.put(
        format!("inbox/items/{name}/brief.md"),
        "<!-- format: weir-brief/1 -->\nthe brief\n",
    );
    let send = SendRecord {
        item: name.into(),
        target: "agentplot/weir fix".into(),
        brief: Some("the brief\n".into()),
        sent_at: "2026-10-25T10:00:00Z".into(),
        outcome: Outcome::Open,
    };
    write.put(
        format!("inbox/items/{name}/sends.rec"),
        weir_capture::rec::write(&[send.to_record()]),
    );
    after(inbox, &write)
}

#[test]
fn an_attach_to_an_item_not_sent() {
    let inbox = moved(
        &three(),
        &a_move(
            "release/x#1",
            "join",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ),
    );
    let not = refused(
        &inbox,
        &a_move(
            "release/x#2",
            "attach",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ),
    );
    assert_eq!(not.field.as_deref(), Some("target"));
    assert!(
        not.why.contains("`shim-timeout`, which is not sent"),
        "{}",
        not.why
    );
    let not = refused(
        &inbox,
        &a_move(
            "release/x#2",
            "attach",
            "no-such-item",
            "curation",
            CURATION_AT,
        ),
    );
    assert!(not.why.contains("no item of the inbox is"), "{}", not.why);
    // Once sent, an attach adds to it.
    let inbox = sent(&inbox, "shim-timeout");
    let inbox = moved(
        &inbox,
        &a_move(
            "release/x#2",
            "attach",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ),
    );
    assert_eq!(
        inbox.standing(&signal("release/x#2")).unwrap().1.word,
        Word::Attach
    );
}

#[test]
fn a_join_to_a_sent_item() {
    let inbox = moved(
        &three(),
        &a_move(
            "release/x#1",
            "join",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ),
    );
    let inbox = sent(&inbox, "shim-timeout");
    assert!(inbox.item("shim-timeout").unwrap().is_sent());
    let not = refused(
        &inbox,
        &a_move(
            "release/x#2",
            "join",
            "shim-timeout",
            "curation",
            CURATION_AT,
        ),
    );
    assert_eq!(not.field.as_deref(), Some("target"));
    assert_eq!(
        not.why,
        "names `shim-timeout`, which is sent: an attach is the move that adds to a sent item"
    );
}

#[test]
fn a_nested_name() {
    let not = refused(
        &three(),
        &a_move(
            "release/x#1",
            "join",
            "shim/timeout",
            "curation",
            CURATION_AT,
        ),
    );
    assert_eq!(not.field.as_deref(), Some("target"));
    assert!(not.why.contains("`shim/timeout`"), "{}", not.why);
    for name in [".", ".."] {
        assert!(three()
            .make_move(&a_move(
                "release/x#1",
                "join",
                name,
                "curation",
                CURATION_AT
            ))
            .is_err());
    }
}

#[test]
fn every_word_reads_back_as_written() {
    let mut inbox = with_signals(&empty(), "release/x", 6);
    let inbox_with_item = moved(
        &inbox,
        &a_move("release/x#6", "join", "pin-mise", "curation", CURATION_AT),
    );
    inbox = sent(&inbox_with_item, "pin-mise");
    let mut written = vec![inbox.moves()[0].clone()];
    for (n, (word, target)) in [
        ("drop", "noise"),
        ("answered", "claim: the pin is current"),
        ("challenge", "claim: the shim is fixed\nin 2026.10.10"),
        ("join", "shim-timeout"),
        ("attach", "pin-mise"),
    ]
    .into_iter()
    .enumerate()
    {
        let new = a_move(
            &format!("release/x#{}", n + 1),
            word,
            target,
            "curation",
            CURATION_AT,
        );
        inbox = moved(&inbox, &new);
        written.push(Move {
            signal: new.signal,
            word: Word::parse(word).unwrap(),
            target: target.into(),
            by: "curation".into(),
            at: CURATION_AT.into(),
            batch: None,
            replaces: None,
        });
    }
    assert_eq!(inbox.moves(), written.as_slice());
    let again = InboxAt::read(inbox.files().iter()).unwrap();
    assert_eq!(again, inbox);
}

#[test]
fn a_tip_whose_moves_break_the_rules_is_refused() {
    let inbox = three();
    let drop = "format: weir-move/1\nsignal: release/x#1\nmove: drop\ntarget: noise\n\
                by: curation\nat: 2026-10-25T09:00:00Z\n";
    for (moves, field) in [
        // A move for a signal the inbox does not hold.
        (drop.replace("release/x#1", "release/x#9"), Some("signal")),
        // A second move that replaces nothing.
        (format!("{drop}\n{drop}"), None),
        // A replacement naming no standing move.
        (format!("{drop}\n{drop}replaces: 2\n"), Some("replaces")),
        // A join naming no item.
        (drop.replace("move: drop", "move: join"), Some("target")),
        // A word not of the six.
        (drop.replace("move: drop", "move: promote"), Some("move")),
    ] {
        let mut files = inbox.files().clone();
        files.insert("inbox/moves/moves.rec".into(), moves.clone());
        let error = InboxAt::read(files.iter()).unwrap_err();
        assert_eq!(error.path, "inbox/moves/moves.rec", "{moves}");
        assert_eq!(error.field.as_deref(), field, "{error}");
    }
}
