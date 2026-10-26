//! The import operation, held to the scenarios of inbox-import: captures
//! read before their inbox existed, written as the body gave them, held by
//! key like any other, taken whole or not at all, in one write with its
//! report (37 to 39, 63, 75a; S8, S9).

mod common;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{take, Bank, Inbox, RawFile, Refusal, Take, Taken};
use weir_capture::{CameIn, Key};
use weir_signals::{
    Imported, ImportedCapture, ImportedMove, ImportedSignal, InboxAt, ReadSignal, Word,
};

use common::{after, empty, key, recsel_value, signal};

const BODY: &str = "import:crew-folders";
const AT: &str = "2026-10-20T08:00:00Z";

fn imported_at() -> OffsetDateTime {
    OffsetDateTime::parse("2026-10-25T11:30:00+02:00", &Rfc3339).unwrap()
}

fn read_signal(n: u64, kind: &str) -> ImportedSignal {
    ImportedSignal {
        n,
        signal: ReadSignal {
            kind: kind.into(),
            asserted_by: "the operator".into(),
            assertion: format!("assertion {n}"),
            subjects: vec!["mise".into()],
            argues_with: Vec::new(),
            excerpt: format!("words {n}, as the body read them, standing in no bank"),
            position: format!("signal {n} of the folder"),
        },
    }
}

/// An imported capture under `key` with `signals` signals and no moves.
fn line(key: &str, signals: u64) -> ImportedCapture {
    ImportedCapture {
        key: key.into(),
        source: key.split('/').next().unwrap_or_default().into(),
        event_at: "2026-10-10T14:02:11Z".into(),
        title: Some(format!("{key}, the title")),
        url: None,
        pointer: Some(format!("mac-studio:~/.local/state/crew/{key}")),
        came_in: CameIn::Import("crew-folders".into()),
        signals: (1..=signals).map(|n| read_signal(n, "ask")).collect(),
        moves: Vec::new(),
    }
}

fn a_move(signal: u64, word: &str, target: &str) -> ImportedMove {
    ImportedMove {
        signal,
        word: word.into(),
        target: target.into(),
        by: "the operator".into(),
        at: AT.into(),
    }
}

fn import(inbox: &InboxAt, captures: &[ImportedCapture]) -> Imported {
    inbox
        .import(captures, "deneb", imported_at())
        .unwrap_or_else(|refused| panic!("{refused}"))
}

/// The inbox after `captures` are imported into `inbox`.
fn imported(inbox: &InboxAt, captures: &[ImportedCapture]) -> InboxAt {
    let write = import(inbox, captures).write.expect("a write");
    after(inbox, &write)
}

#[test]
fn a_release_capture_imported() {
    let mut release = line("release/mise/2026.10.10", 0);
    release.pointer = Some("mac-studio:~/.local/state/crew/x".into());
    let Imported { write, report } = import(&empty(), &[release]);
    let write = write.unwrap();
    assert_eq!(
        write.files.keys().collect::<Vec<_>>(),
        ["inbox/captures/release/mise/2026.10.10/capture.rec"]
    );
    let text = &write.files["inbox/captures/release/mise/2026.10.10/capture.rec"];
    assert_eq!(recsel_value(text, "key"), "release/mise/2026.10.10");
    assert_eq!(
        recsel_value(text, "pointer"),
        "mac-studio:~/.local/state/crew/x"
    );
    assert_eq!(recsel_value(text, "came_in"), BODY);
    assert_eq!(recsel_value(text, "machine"), "deneb");
    assert_eq!(recsel_value(text, "taken_at"), "2026-10-25T09:30:00Z");
    assert_eq!(recsel_value(text, "unread"), "yes");
    assert_eq!(report.written, [key("release/mise/2026.10.10")]);
    let inbox = after(&empty(), &write);
    let capture = inbox.capture(&key("release/mise/2026.10.10")).unwrap();
    assert_eq!(
        capture.title.as_deref(),
        Some("release/mise/2026.10.10, the title")
    );
    assert_eq!(capture.event_at, "2026-10-10T14:02:11Z");
    assert!(inbox.is_unread(&key("release/mise/2026.10.10")));
}

#[test]
fn a_capture_with_no_pointer() {
    let mut no_pointer = line("meeting/file/ab12", 0);
    no_pointer.pointer = None;
    let inbox = imported(&empty(), &[no_pointer]);
    let capture = inbox.capture(&key("meeting/file/ab12")).unwrap();
    assert_eq!(capture.pointer, None);
    assert!(!inbox.files()["inbox/captures/meeting/file/ab12/capture.rec"].contains("pointer"));
}

#[test]
fn signals_and_moves_as_read() {
    let mut three = line("release/x", 3);
    three.moves = vec![a_move(2, "drop", "a duplicate of release/y")];
    let inbox = imported(&empty(), &[three]);
    let signals = inbox.signals(&key("release/x")).unwrap();
    let ids: Vec<String> = signals.iter().map(|s| s.signal.to_string()).collect();
    assert_eq!(ids, ["release/x#1", "release/x#2", "release/x#3"]);
    assert_eq!(signals[1].assertion, "assertion 2");
    assert!(!inbox.is_unread(&key("release/x")));
    let (_, standing) = inbox.standing(&signal("release/x#2")).unwrap();
    assert_eq!(standing.word, Word::Drop);
    assert_eq!(standing.target, "a duplicate of release/y");
    assert_eq!(standing.by, "the operator");
    assert_eq!(standing.at, AT);
    assert_eq!(inbox.standing(&signal("release/x#1")), None);
}

#[test]
fn an_excerpt_found_in_nothing() {
    let inbox = imported(&empty(), &[line("release/x", 1)]);
    assert_eq!(
        inbox.signal(&signal("release/x#1")).unwrap().excerpt,
        "words 1, as the body read them, standing in no bank"
    );
}

#[test]
fn one_body_imported_twice() {
    // S8: the second import, on the tip the first left, writes nothing.
    let mut body = vec![
        line("release/a", 2),
        line("meeting/file/ab12", 0),
        line("folder/x/1", 1),
    ];
    body[0].moves = vec![a_move(1, "join", "pin-mise")];
    let inbox = imported(&empty(), &body);
    let Imported { write, report } = import(&inbox, &body);
    assert_eq!(write, None);
    assert_eq!(report.held, 3);
    assert!(report.written.is_empty());
    assert!(report.refused.is_empty());
    assert!(report.not_moved.is_empty());
}

/// An inbox a host's `take` writes through, read again after each write.
struct Tip(InboxAt);

impl Inbox for Tip {
    fn holds(&self, key: &Key) -> bool {
        self.0.holds(key)
    }

    fn write(&mut self, taken: &Taken) -> Result<(), Refusal> {
        let write = self.0.take(taken).map_err(|r| Refusal(r.to_string()))?;
        self.0 = after(&self.0, &write);
        Ok(())
    }
}

/// A bank no test here reaches: a held offer banks nothing (14).
struct NoBank;

impl Bank for NoBank {
    fn bank(&mut self, key: &Key, _: &[RawFile], _: Option<&str>) -> Result<String, Refusal> {
        panic!("{key} was banked")
    }

    fn discard(&mut self, key: &Key) {
        panic!("{key} was discarded")
    }
}

#[test]
fn an_offer_under_an_imported_key() {
    let inbox = imported(&empty(), &[line("release/mise/2026.10.10", 1)]);
    let offer = r#"{"capture": {"key": "release/mise/2026.10.10", "source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#;
    let how = Take::new("deneb", imported_at(), 8192, CameIn::Run("releases".into())).unwrap();
    let mut tip = Tip(inbox.clone());
    let report = take([offer], None, &how, &mut tip, &mut NoBank);
    assert_eq!(report.held, 1);
    assert!(report.taken.is_empty());
    assert_eq!(tip.0, inbox);
    // The inbox's own take holds it as it holds any other.
    let refused = inbox
        .take(&Taken {
            capture: common::capture("release/mise/2026.10.10"),
            own_signal: None,
        })
        .unwrap_err();
    assert!(refused.0[0].why.contains("is held"), "{refused}");
}

#[test]
fn a_capture_that_names_no_key() {
    let lines = [line("", 0), line("moves/x", 0), line("release/x", 0)];
    let Imported { write, report } = import(&empty(), &lines);
    assert_eq!(report.written, [key("release/x")]);
    let refused: Vec<(&str, &str)> = report
        .refused
        .iter()
        .map(|r| (r.key.as_str(), r.field.as_str()))
        .collect();
    assert_eq!(refused, [("", "key"), ("moves/x", "key")]);
    assert!(report.refused[0].why.contains("names no key"), "{report:?}");
    assert!(
        report.refused[1]
            .why
            .contains("first segment is never `moves`"),
        "{report:?}"
    );
    assert_eq!(write.unwrap().files.len(), 1);
}

#[test]
fn a_capture_out_of_shape_is_refused_whole() {
    let mut no_source = line("release/a", 1);
    no_source.source = String::new();
    no_source.moves = vec![a_move(1, "drop", "noise")];
    let mut no_offset = line("release/b", 0);
    no_offset.event_at = "2026-10-10T14:02:11".into();
    let mut by_a_run = line("release/c", 0);
    by_a_run.came_in = CameIn::Run("releases".into());
    let mut a_gap = line("release/d", 2);
    a_gap.signals[1].n = 3;
    let Imported { write, report } = import(&empty(), &[no_source, no_offset, by_a_run, a_gap]);
    assert_eq!(write, None);
    let refused: Vec<(&str, &str)> = report
        .refused
        .iter()
        .map(|r| (r.key.as_str(), r.field.as_str()))
        .collect();
    assert_eq!(
        refused,
        [
            ("release/a", "source"),
            ("release/b", "event_at"),
            ("release/c", "came_in"),
            ("release/d", "n"),
        ]
    );
    // A refused capture's moves go with it, unreported.
    assert!(report.not_moved.is_empty());
}

#[test]
fn one_bad_capture_among_many() {
    let mut lines: Vec<ImportedCapture> = (1..=20)
        .map(|n| line(&format!("release/r{n}"), 2))
        .collect();
    lines[6].signals[1].signal.kind = String::new();
    let Imported { write, report } = import(&empty(), &lines);
    assert_eq!(report.written.len(), 19);
    assert!(!report.written.contains(&key("release/r7")));
    assert_eq!(report.refused.len(), 1);
    let refused = &report.refused[0];
    assert_eq!(refused.key, "release/r7");
    assert_eq!(refused.signal, Some(signal("release/r7#2")));
    assert_eq!(refused.field, "kind");
    let inbox = after(&empty(), &write.unwrap());
    assert!(!inbox.holds(&key("release/r7")));
    assert_eq!(inbox.keys().count(), 19);
    // So is a nature not of the five.
    let mut complaint = line("release/z", 1);
    complaint.signals[0].signal.kind = "complaint".into();
    let report = import(&empty(), &[complaint]).report;
    assert_eq!(report.refused[0].field, "kind");
    assert!(report.refused[0].why.contains("`complaint`"), "{report:?}");
}

#[test]
fn the_report() {
    let inbox = imported(&empty(), &[line("release/held", 0)]);
    let mut promoted = line("release/c", 1);
    promoted.moves = vec![a_move(1, "promote", "the roadmap")];
    let lines = [
        line("release/a", 0),
        line("release/held", 1),
        line("release/b", 1),
        promoted,
        line("meeting/file/ab12", 2),
    ];
    let Imported { write, report } = import(&inbox, &lines);
    let write = write.unwrap();
    let captures: Vec<&String> = write
        .files
        .keys()
        .filter(|path| path.ends_with("capture.rec"))
        .collect();
    assert_eq!(captures.len(), 4);
    assert_eq!(
        report.written,
        [
            key("release/a"),
            key("release/b"),
            key("release/c"),
            key("meeting/file/ab12"),
        ]
    );
    assert_eq!(report.held, 1);
    assert!(report.refused.is_empty());
    assert_eq!(report.not_moved.len(), 1);
    assert_eq!(report.not_moved[0].signal, signal("release/c#1"));
    assert_eq!(report.not_moved[0].word, "promote");
    let inbox = after(&inbox, &write);
    assert_eq!(inbox.keys().count(), 5);
}

#[test]
fn the_machine_is_one_a_pointer_can_name() {
    for machine in ["", "mac studio", "deneb:1"] {
        assert!(empty()
            .import(&[line("release/x", 0)], machine, imported_at())
            .is_err());
    }
}

/// The moves `report` did not take, each as its signal, word and field.
fn not_moved(report: &weir_signals::ImportReport) -> Vec<(String, String, Option<String>)> {
    report
        .not_moved
        .iter()
        .map(|m| (m.signal.to_string(), m.word.clone(), m.field.clone()))
        .collect()
}

#[test]
fn a_word_not_of_the_six() {
    // S9, with a word a body of another tool's might carry.
    let mut capture = line("release/x", 2);
    capture.moves = vec![
        a_move(1, "promote", "the roadmap"),
        a_move(2, "new-territory", "pin-mise"),
    ];
    let Imported { write, report } = import(&empty(), &[capture]);
    assert_eq!(
        not_moved(&report),
        [
            ("release/x#1".into(), "promote".into(), Some("move".into())),
            (
                "release/x#2".into(),
                "new-territory".into(),
                Some("move".into())
            ),
        ]
    );
    assert!(
        report.not_moved[0].why.contains("not one of drop"),
        "{report:?}"
    );
    let write = write.unwrap();
    assert!(!write.files.contains_key("inbox/moves/moves.rec"));
    let inbox = after(&empty(), &write);
    assert_eq!(inbox.standing(&signal("release/x#1")), None);
    assert_eq!(inbox.standing(&signal("release/x#2")), None);
    assert_eq!(inbox.unmoved().len(), 2);
}

#[test]
fn a_join_with_no_item() {
    let mut capture = line("release/x", 2);
    capture.moves = vec![a_move(1, "join", ""), a_move(2, "drop", "")];
    let report = import(&empty(), &[capture]).report;
    assert_eq!(
        not_moved(&report),
        [
            ("release/x#1".into(), "join".into(), Some("target".into())),
            ("release/x#2".into(), "drop".into(), Some("target".into())),
        ]
    );
    assert!(report.not_moved[0]
        .why
        .contains("the item it makes or grows"));
}

#[test]
fn an_imported_move_is_checked_as_any_move() {
    // An inbox holding a sent item and a proposed one.
    let inbox = imported(
        &empty(),
        &[{
            let mut c = line("release/held", 2);
            c.moves = vec![
                a_move(1, "join", "sent-item"),
                a_move(2, "join", "proposed-item"),
            ];
            c
        }],
    );
    let inbox = after(
        &inbox,
        &inbox.write_brief("sent-item", "the brief\n").unwrap(),
    );
    let inbox = after(
        &inbox,
        &inbox
            .send(&["sent-item"], "agentplot/weir fix", AT)
            .unwrap(),
    );
    let mut capture = line("release/x", 6);
    capture.moves = vec![
        // A signal its capture lacks.
        a_move(7, "drop", "noise"),
        // A second move for one signal.
        a_move(1, "drop", "noise"),
        a_move(1, "route", "an ask"),
        // A join naming a sent item, an attach naming one not sent.
        a_move(2, "join", "sent-item"),
        a_move(3, "attach", "proposed-item"),
        // An attach to the sent item and a join growing the proposed one.
        a_move(4, "attach", "sent-item"),
        a_move(5, "join", "proposed-item"),
        // A move naming no time.
        ImportedMove {
            at: "yesterday".into(),
            ..a_move(6, "drop", "noise")
        },
    ];
    let Imported { write, report } = import(&inbox, &[capture]);
    assert_eq!(
        not_moved(&report),
        [
            ("release/x#7".into(), "drop".into(), Some("signal".into())),
            ("release/x#1".into(), "route".into(), None),
            ("release/x#2".into(), "join".into(), Some("target".into())),
            ("release/x#3".into(), "attach".into(), Some("target".into())),
            ("release/x#6".into(), "drop".into(), Some("at".into())),
        ]
    );
    assert!(
        report.not_moved[1].why.contains("standing move"),
        "{report:?}"
    );
    assert!(
        report.not_moved[2].why.contains("which is sent"),
        "{report:?}"
    );
    assert!(report.not_moved[3].why.contains("not sent"), "{report:?}");
    let inbox = after(&inbox, &write.unwrap());
    assert_eq!(
        inbox.standing(&signal("release/x#1")).unwrap().1.word,
        Word::Drop
    );
    for n in [2, 3, 6] {
        assert_eq!(inbox.standing(&signal(&format!("release/x#{n}"))), None);
    }
    let ids = |name: &str| -> Vec<String> {
        inbox
            .item_signals(name)
            .iter()
            .map(|id| id.to_string())
            .collect()
    };
    assert_eq!(ids("sent-item"), ["release/held#1", "release/x#4"]);
    assert_eq!(ids("proposed-item"), ["release/held#2", "release/x#5"]);
}

#[test]
fn two_joins_into_one_item() {
    let mut first = line("release/a", 1);
    first.moves = vec![a_move(1, "join", "pin-mise")];
    let mut second = line("meeting/file/ab12", 1);
    second.moves = vec![ImportedMove {
        by: "curation".into(),
        ..a_move(1, "join", "pin-mise")
    }];
    let Imported { write, report } = import(&empty(), &[first, second]);
    assert!(report.not_moved.is_empty(), "{report:?}");
    let write = write.unwrap();
    let items: Vec<&String> = write
        .files
        .keys()
        .filter(|path| path.starts_with("inbox/items/"))
        .collect();
    assert_eq!(items, ["inbox/items/pin-mise/item.rec"]);
    let inbox = after(&empty(), &write);
    let item = inbox.item("pin-mise").unwrap();
    assert_eq!(item.item.made_by, weir_signals::MadeBy::Join);
    assert_eq!(item.item.named_by, "the operator");
    assert_eq!(item.item.named_at, AT);
    assert_eq!(item.brief, None);
    assert!(!item.is_sent());
    let ids: Vec<String> = inbox
        .item_signals("pin-mise")
        .iter()
        .map(|id| id.to_string())
        .collect();
    assert_eq!(ids, ["meeting/file/ab12#1", "release/a#1"]);
}
