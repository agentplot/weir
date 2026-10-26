//! Every record of every kind, formed by its operation: each names its
//! format first, every `.rec` file passes `recfix`, and each reads back as
//! it was written (40; core.22, core.24).

mod common;

use weir_capture::rec;
use weir_capture::{LeftOut, Nature, OwnSignal, RawFile, Taken};
use weir_signals::format::Format;
use weir_signals::{
    Decision, InboxAt, Item, MadeBy, Move, NewMove, Outcome, ReadSignal, SendRecord, Signal,
    SignalId, Word, Write,
};

use common::{a_move, capture, empty, key, recfix, signal};

const AT: &str = "2026-10-25T09:00:00Z";

/// Holds `write` to every rule of a formed write: each `.rec` file passes
/// `recfix` and each of its records names its format first; each brief
/// names its format on its first line. Gives the inbox after it.
fn formed(inbox: &InboxAt, write: Write) -> InboxAt {
    for (path, text) in &write.files {
        if path.ends_with(".rec") {
            recfix(text);
            for record in rec::read(text).unwrap() {
                let (name, value) = &record.fields()[0];
                assert_eq!(name, "format", "{path}: {record:?}");
                assert!(
                    Format::ALL.iter().any(|format| format.as_str() == value),
                    "{path}: {value}"
                );
            }
        } else {
            assert!(path.ends_with("/brief.md"), "{path}");
            assert!(
                text.starts_with("<!-- format: weir-brief/1 -->\n"),
                "{path}: {text}"
            );
        }
    }
    inbox
        .check(&write)
        .unwrap_or_else(|refused| panic!("{refused}"))
}

fn read_signal(excerpt: &str, kind: &str) -> ReadSignal {
    ReadSignal {
        kind: kind.into(),
        asserted_by: "Ana".into(),
        assertion: format!("{excerpt}, asserted"),
        subjects: vec!["mise".into(), "shim".into()],
        argues_with: Vec::new(),
        excerpt: excerpt.into(),
        position: "the transcript, line 2".into(),
    }
}

#[test]
fn every_record_written_by_its_operation_reads_back_as_written() {
    let mut inbox = empty();

    // A capture with its cut own signal, and an unread capture.
    let own = OwnSignal {
        signal: "release/mise/2026.10.10#1".into(),
        kind: Nature::Ask,
        asserted_by: "jdx/mise".into(),
        assertion: "mise 2026.10.10 is released".into(),
        subjects: vec!["release".into()],
        argues_with: vec!["packages/mise/default.nix".into()],
        position: "release notes".into(),
        excerpt: "## Fixes\n[cut: 2 lines and 30 bytes left out, banked whole with the capture]"
            .into(),
        cut: Some(LeftOut {
            lines: 2,
            bytes: 30,
        }),
    };
    let release = Taken {
        capture: weir_capture::Capture {
            unread: false,
            ..capture("release/mise/2026.10.10")
        },
        own_signal: Some(own.clone()),
    };
    inbox = formed(&inbox, inbox.take(&release).unwrap());
    let meeting = Taken {
        capture: capture("meeting/file/ab12"),
        own_signal: None,
    };
    inbox = formed(&inbox, inbox.take(&meeting).unwrap());
    let empty_reading = Taken {
        capture: capture("folder/notes/cd34"),
        own_signal: None,
    };
    inbox = formed(&inbox, inbox.take(&empty_reading).unwrap());

    // A reading of three, and a reading of none.
    let transcript = "Ana: the shim times out\\\nBen: we pin mise\nAna: we ask upstream\n";
    let raw = RawFile {
        path: "transcript.txt".into(),
        bytes: transcript.as_bytes().to_vec(),
    };
    let reading = [
        read_signal("the shim times out\\", "constraint"),
        read_signal("we pin mise", "commitment"),
        read_signal("Ana: we ask upstream\n", "question"),
    ];
    let meeting_key = key("meeting/file/ab12");
    inbox = formed(
        &inbox,
        inbox.read_capture(&meeting_key, &reading, &[raw]).unwrap(),
    );
    inbox = formed(
        &inbox,
        inbox
            .read_capture(&key("folder/notes/cd34"), &[], &[])
            .unwrap(),
    );

    // Moves of every word, one making an item and one a replacement.
    let moves = [
        a_move(
            "meeting/file/ab12#1",
            "join",
            "shim-timeout",
            "curation",
            AT,
        ),
        a_move(
            "meeting/file/ab12#2",
            "challenge",
            "claim: the pin is\ncurrent",
            "curation",
            AT,
        ),
        NewMove {
            replaces: Some(1),
            ..a_move(
                "meeting/file/ab12#2",
                "answered",
                "record: inquiry/toolchain#2",
                "operator",
                "2026-10-25T11:00:00+02:00",
            )
        },
    ];
    for new in &moves {
        inbox = formed(&inbox, inbox.make_move(new).unwrap());
    }

    // An item from a conclusion, briefed, sent and settled.
    inbox = formed(
        &inbox,
        inbox
            .propose_item("pin-mise", "inquiry/toolchain#3", "operator", AT)
            .unwrap(),
    );
    let brief = "# Pin mise\n\nUntil the shim is fixed.\\\n";
    inbox = formed(&inbox, inbox.write_brief("pin-mise", brief).unwrap());
    inbox = formed(
        &inbox,
        inbox
            .send(&["pin-mise"], "agentplot/weir fix", "2026-10-25T12:00:00Z")
            .unwrap(),
    );
    let landing = Outcome::Landing("agentplot/weir 1a2b3c4".into());
    inbox = formed(
        &inbox,
        inbox
            .settle_send(
                "pin-mise",
                "agentplot/weir fix",
                "2026-10-25T12:00:00Z",
                &landing,
            )
            .unwrap(),
    );

    // Batches: one standing with a join and an attach, one refused.
    let batch_moves = [
        a_move("meeting/file/ab12#3", "attach", "pin-mise", "curation", AT),
        a_move(
            "release/mise/2026.10.10#1",
            "route",
            "an ask upstream",
            "curation",
            AT,
        ),
    ];
    inbox = formed(
        &inbox,
        inbox
            .propose_batch("2026-10-25-a", "curation", AT, &batch_moves)
            .unwrap(),
    );
    inbox = formed(
        &inbox,
        inbox
            .decide_batch("2026-10-25-a", true, "operator", AT)
            .unwrap(),
    );
    inbox = formed(
        &inbox,
        inbox
            .propose_batch(
                "2026-10-25-b",
                "curation",
                AT,
                &[NewMove {
                    replaces: Some(1),
                    ..a_move("meeting/file/ab12#1", "drop", "noise", "curation", AT)
                }],
            )
            .unwrap(),
    );
    inbox = formed(
        &inbox,
        inbox
            .decide_batch("2026-10-25-b", false, "operator", AT)
            .unwrap(),
    );

    // The inbox read again from its files is the inbox the writes left.
    let again = InboxAt::read(inbox.files().iter()).unwrap();
    assert_eq!(again, inbox);

    // And each record reads back as it was written.
    let read = &again;
    assert_eq!(
        read.capture(&key("release/mise/2026.10.10")),
        Some(&release.capture)
    );
    assert_eq!(read.capture(&meeting_key), Some(&meeting.capture));
    assert_eq!(
        read.signals(&key("release/mise/2026.10.10")).unwrap(),
        [Signal::own(&own).unwrap()]
    );
    let signals = read.signals(&meeting_key).unwrap();
    for (n, (signal, handed)) in (1..).zip(signals.iter().zip(&reading)) {
        let expected = handed
            .to_signal(SignalId::new(meeting_key.clone(), n))
            .unwrap();
        assert_eq!(signal, &expected);
    }
    assert_eq!(read.signals(&key("folder/notes/cd34")), Some(&[][..]));
    let as_written: Vec<Move> = moves
        .iter()
        .map(|new| Move {
            signal: new.signal.clone(),
            word: Word::parse(&new.word).unwrap(),
            target: new.target.clone(),
            by: new.by.clone(),
            at: new.at.clone(),
            batch: None,
            replaces: new.replaces,
        })
        .chain(batch_moves.iter().map(|new| Move {
            signal: new.signal.clone(),
            word: Word::parse(&new.word).unwrap(),
            target: new.target.clone(),
            by: new.by.clone(),
            at: new.at.clone(),
            batch: Some("2026-10-25-a".into()),
            replaces: None,
        }))
        .collect();
    assert_eq!(read.moves(), as_written.as_slice());
    assert_eq!(
        read.item("shim-timeout").unwrap().item,
        Item {
            item: "shim-timeout".into(),
            made_by: MadeBy::Join,
            named_by: "curation".into(),
            named_at: AT.into(),
        }
    );
    let pin = read.item("pin-mise").unwrap();
    assert_eq!(
        pin.item.made_by,
        MadeBy::Conclusion("inquiry/toolchain#3".into())
    );
    assert_eq!(pin.brief.as_deref(), Some(brief));
    assert_eq!(
        pin.sends,
        [
            SendRecord {
                item: "pin-mise".into(),
                target: "agentplot/weir fix".into(),
                brief: Some(brief.into()),
                sent_at: "2026-10-25T12:00:00Z".into(),
                outcome: Outcome::Open,
            },
            SendRecord {
                item: "pin-mise".into(),
                target: "agentplot/weir fix".into(),
                brief: None,
                sent_at: "2026-10-25T12:00:00Z".into(),
                outcome: landing,
            },
        ]
    );
    let standing = read.batch("2026-10-25-a").unwrap();
    assert_eq!(standing.proposal.proposed_by, "curation");
    assert_eq!(standing.moves, as_written[3..]);
    assert_eq!(
        standing.decision,
        Some(Decision {
            batch: "2026-10-25-a".into(),
            stands: true,
            decided_by: "operator".into(),
            decided_at: AT.into(),
        })
    );
    assert_eq!(read.batch("2026-10-25-b").unwrap().stands(), Some(false));
    assert_eq!(
        read.standing(&signal("meeting/file/ab12#1"))
            .unwrap()
            .1
            .word,
        Word::Join
    );
}
