//! Batches, held to the scenarios of inbox-signals: proposed whole, their
//! moves not standing until the batch stands whole, or refused whole
//! (31, 32; core.19).

mod common;

use weir_signals::{Decision, InboxAt, NewMove, Word, Write};

use common::{a_move, after, empty, moved, paths, recsel_value, signal, with_signals};

const PROPOSED_AT: &str = "2026-10-25T09:00:00Z";
const DECIDED_AT: &str = "2026-10-25T12:30:00+02:00";

/// An inbox holding `release/x` read into four signals.
fn four() -> InboxAt {
    with_signals(&empty(), "release/x", 4)
}

/// Four moves, one for each of `release/x`'s signals.
fn four_moves() -> Vec<NewMove> {
    vec![
        a_move("release/x#1", "drop", "noise", "curation", PROPOSED_AT),
        a_move(
            "release/x#2",
            "join",
            "shim-timeout",
            "curation",
            PROPOSED_AT,
        ),
        a_move(
            "release/x#3",
            "join",
            "shim-timeout",
            "curation",
            PROPOSED_AT,
        ),
        a_move(
            "release/x#4",
            "route",
            "an ask upstream",
            "curation",
            PROPOSED_AT,
        ),
    ]
}

/// `four()` with batch `2026-10-25-a` of `four_moves()` proposed.
fn proposed() -> InboxAt {
    let inbox = four();
    let write = inbox
        .propose_batch("2026-10-25-a", "curation", PROPOSED_AT, &four_moves())
        .unwrap();
    after(&inbox, &write)
}

#[test]
fn a_proposed_batch() {
    let inbox = four();
    let write = inbox
        .propose_batch("2026-10-25-a", "curation", PROPOSED_AT, &four_moves())
        .unwrap();
    assert_eq!(paths(&write), ["inbox/moves/batches/2026-10-25-a.rec"]);
    let text = &write.files["inbox/moves/batches/2026-10-25-a.rec"];
    assert!(
        text.starts_with(
            "format: weir-batch/1\n\
             batch: 2026-10-25-a\n\
             proposed_by: curation\n\
             proposed_at: 2026-10-25T09:00:00Z\n\
             \n\
             format: weir-move/1\n\
             signal: release/x#1\n\
             move: drop\n\
             target: noise\n\
             by: curation\n\
             at: 2026-10-25T09:00:00Z\n\
             batch: 2026-10-25-a\n"
        ),
        "{text}"
    );
    let inbox = after(&inbox, &write);
    for n in 1..=4 {
        assert_eq!(inbox.standing(&signal(&format!("release/x#{n}"))), None);
    }
    let batch = inbox.batch("2026-10-25-a").unwrap();
    assert_eq!(batch.stands(), None);
    assert_eq!(batch.moves.len(), 4);
    assert!(batch
        .moves
        .iter()
        .all(|m| m.batch.as_deref() == Some("2026-10-25-a")));
    // No item is made while the batch is proposed.
    assert_eq!(inbox.item("shim-timeout"), None);
    assert!(inbox.moves().is_empty());
}

#[test]
fn a_batch_name_used_before() {
    let inbox = proposed();
    let refused = inbox
        .propose_batch(
            "2026-10-25-a",
            "curation",
            PROPOSED_AT,
            &[a_move(
                "release/x#1",
                "drop",
                "noise",
                "curation",
                PROPOSED_AT,
            )],
        )
        .unwrap_err();
    assert_eq!(refused.0[0].what, "2026-10-25-a");
    assert!(refused.0[0].why.contains("has had"), "{refused}");
    // Nor is a name with a `/`, or no name, a batch's.
    for name in ["2026/10/25", "", ".."] {
        assert!(four()
            .propose_batch(name, "curation", PROPOSED_AT, &four_moves())
            .is_err());
    }
    // A batch is some moves, proposed by someone at a time.
    assert!(four()
        .propose_batch("b", "curation", PROPOSED_AT, &[])
        .is_err());
    let refused = four()
        .propose_batch("b", "", PROPOSED_AT, &four_moves())
        .unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("proposed_by"));
    let refused = four()
        .propose_batch("b", "curation", "today", &four_moves())
        .unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("proposed_at"));
}

#[test]
fn a_batch_is_proposed_whole_or_refused_whole() {
    let inbox = moved(
        &four(),
        &a_move("release/x#4", "drop", "noise", "operator", PROPOSED_AT),
    );
    let moves = [
        a_move("release/x#1", "drop", "noise", "curation", PROPOSED_AT),
        a_move(
            "release/x#2",
            "promote",
            "the roadmap",
            "curation",
            PROPOSED_AT,
        ),
        a_move(
            "release/x#1",
            "join",
            "shim-timeout",
            "curation",
            PROPOSED_AT,
        ),
        a_move("release/x#3", "challenge", "", "curation", PROPOSED_AT),
        a_move("release/x#4", "route", "an ask", "curation", PROPOSED_AT),
    ];
    let refused = inbox
        .propose_batch("2026-10-25-b", "curation", PROPOSED_AT, &moves)
        .unwrap_err();
    let named: Vec<(&str, Option<&str>)> = refused
        .0
        .iter()
        .map(|n| (n.what.as_str(), n.field.as_deref()))
        .collect();
    assert_eq!(
        named,
        [
            ("release/x#2", Some("move")),
            ("release/x#1", None),
            ("release/x#3", Some("target")),
            ("release/x#4", None),
        ]
    );
    assert!(refused.0[1].why.contains("moved twice"), "{refused}");
    assert!(refused.0[3].why.contains("standing move"), "{refused}");
}

#[test]
fn a_batch_stands() {
    let inbox = proposed();
    let write = inbox
        .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
        .unwrap();
    assert_eq!(
        paths(&write),
        [
            "inbox/items/shim-timeout/item.rec",
            "inbox/moves/batches/2026-10-25-a.rec",
            "inbox/moves/moves.rec",
        ]
    );
    let file = &write.files["inbox/moves/batches/2026-10-25-a.rec"];
    let before = &inbox.files()["inbox/moves/batches/2026-10-25-a.rec"];
    assert!(file.starts_with(before.as_str()));
    assert_eq!(
        &file[before.len()..],
        "\nformat: weir-batch/1\n\
         batch: 2026-10-25-a\n\
         stands: yes\n\
         decided_by: operator\n\
         decided_at: 2026-10-25T12:30:00+02:00\n"
    );
    let inbox = after(&inbox, &write);
    assert_eq!(inbox.moves().len(), 4);
    for (n, word) in (1..).zip([Word::Drop, Word::Join, Word::Join, Word::Route]) {
        let (_, standing) = inbox.standing(&signal(&format!("release/x#{n}"))).unwrap();
        assert_eq!(standing.word, word);
        assert_eq!(standing.batch.as_deref(), Some("2026-10-25-a"));
    }
    let batch = inbox.batch("2026-10-25-a").unwrap();
    assert_eq!(batch.stands(), Some(true));
    assert_eq!(
        batch.decision,
        Some(Decision {
            batch: "2026-10-25-a".into(),
            stands: true,
            decided_by: "operator".into(),
            decided_at: DECIDED_AT.into(),
        })
    );
    // The batch's two joins make one item, named by the first join's maker.
    let item = inbox.item("shim-timeout").unwrap();
    assert_eq!(item.item.named_by, "curation");
    assert_eq!(
        recsel_value(&write.files["inbox/items/shim-timeout/item.rec"], "made_by"),
        "join"
    );
}

#[test]
fn one_signal_moved_meanwhile() {
    let inbox = moved(
        &proposed(),
        &a_move("release/x#3", "drop", "duplicate", "operator", PROPOSED_AT),
    );
    let refused = inbox
        .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
        .unwrap_err();
    assert_eq!(refused.0.len(), 1);
    assert_eq!(refused.0[0].what, "release/x#3");
    assert!(
        refused.0[0]
            .why
            .contains("move 1 of `release/x#3`, a drop by operator"),
        "{refused}"
    );
    for n in [1, 2, 4] {
        assert_eq!(inbox.standing(&signal(&format!("release/x#{n}"))), None);
    }
}

#[test]
fn a_batch_refused_by_the_operator() {
    let inbox = proposed();
    let write = inbox
        .decide_batch("2026-10-25-a", false, "operator", DECIDED_AT)
        .unwrap();
    assert_eq!(paths(&write), ["inbox/moves/batches/2026-10-25-a.rec"]);
    let inbox = after(&inbox, &write);
    let batch = inbox.batch("2026-10-25-a").unwrap();
    assert_eq!(batch.stands(), Some(false));
    assert_eq!(batch.decision.as_ref().unwrap().decided_by, "operator");
    assert!(inbox.moves().is_empty());
    // Its moves never stand: it cannot be decided again.
    assert!(inbox
        .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
        .is_err());
    // Its signals stay free for other moves.
    moved(
        &inbox,
        &a_move("release/x#1", "drop", "noise", "operator", DECIDED_AT),
    );
}

#[test]
fn a_batch_decided_twice() {
    let inbox = proposed();
    let inbox = after(
        &inbox,
        &inbox
            .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
            .unwrap(),
    );
    let refused = inbox
        .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
        .unwrap_err();
    assert_eq!(refused.0[0].what, "2026-10-25-a");
    assert_eq!(refused.0[0].field.as_deref(), Some("stands"));
    assert_eq!(
        refused.0[0].why,
        "is decided already: stands: yes, decided by operator at 2026-10-25T12:30:00+02:00"
    );
    assert!(inbox
        .decide_batch("2026-10-25-z", true, "operator", DECIDED_AT)
        .is_err());
}

#[test]
fn a_tip_whose_batches_break_the_rules_is_refused() {
    let inbox = proposed();
    let file = inbox.files()["inbox/moves/batches/2026-10-25-a.rec"].clone();
    let path = "inbox/moves/batches/2026-10-25-a.rec";
    let read = |changes: &[(&str, &str)]| {
        let mut write = Write::new();
        for (path, text) in changes {
            write.put(*path, *text);
        }
        let mut files = inbox.files().clone();
        files.extend(write.files);
        InboxAt::read(files.iter())
    };
    // A file naming another batch, or opening with a move.
    let error = read(&[(
        path,
        &file.replacen("batch: 2026-10-25-a", "batch: other", 1),
    )])
    .unwrap_err();
    assert_eq!(error.field.as_deref(), Some("batch"));
    let moves_first = file.split_once("\n\n").unwrap().1.to_string();
    assert!(read(&[(path, &moves_first)]).is_err());
    // A move in moves.rec naming a batch that does not stand.
    let standing = inbox
        .decide_batch("2026-10-25-a", true, "operator", DECIDED_AT)
        .unwrap();
    let moves = &standing.files["inbox/moves/moves.rec"];
    let error = read(&[("inbox/moves/moves.rec", moves)]).unwrap_err();
    assert_eq!(error.path, "inbox/moves/moves.rec");
    assert_eq!(error.field.as_deref(), Some("batch"), "{error}");
    // A batch that stands with its moves not in moves.rec.
    let decided = &standing.files[path];
    let error = read(&[(path, decided)]).unwrap_err();
    assert_eq!(error.path, path);
    assert!(error.reason.contains("is not in `moves.rec`"), "{error}");
    // A record after the decision.
    let after_decision = format!("{decided}\n{}", file.split_once("\n\n").unwrap().0);
    assert!(read(&[(path, &after_decision)]).is_err());
}
