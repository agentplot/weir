//! Taking lines through a host's inbox and bank, held to the scenarios of
//! capture-take, with in-memory fakes of both.

mod fakes;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{take, Bank, Inbox, RawFile, Refused, Take, TakeError, Taken};
use weir_capture::{CameIn, Capture, Key, SMALLEST_LIMIT};

use fakes::{host, Call};

const LIMIT: usize = 8192;

fn at(time: &str) -> OffsetDateTime {
    OffsetDateTime::parse(time, &Rfc3339).unwrap()
}

fn take_on(machine: &str) -> Take {
    Take::new(
        machine,
        at("2026-10-17T06:30:05+02:00"),
        LIMIT,
        CameIn::Run("releases".into()),
    )
    .unwrap()
}

#[test]
fn a_take_is_made_from_what_the_host_hands_in() {
    let take = take_on("deneb");
    assert_eq!(take.machine(), "deneb");
    assert_eq!(take.taken_at(), "2026-10-17T04:30:05Z");
    assert_eq!(take.limit(), LIMIT);
    assert_eq!(take.came_in(), &CameIn::Run("releases".into()));
}

#[test]
fn a_limit_below_the_widest_closing_line_is_refused() {
    let at = at("2026-10-17T04:30:05Z");
    assert_eq!(
        Take::new("deneb", at, SMALLEST_LIMIT - 1, CameIn::Intake),
        Err(TakeError::Limit(SMALLEST_LIMIT - 1))
    );
    assert!(Take::new("deneb", at, SMALLEST_LIMIT, CameIn::Intake).is_ok());
    for machine in ["", "deneb:1", "de neb"] {
        assert_eq!(
            Take::new(machine, at, LIMIT, CameIn::Intake),
            Err(TakeError::Machine(machine.into()))
        );
    }
}

#[test]
fn the_fakes_record_every_call_in_order() {
    let (mut inbox, mut bank, calls) = host();
    let key = Key::parse("release/x").unwrap();
    inbox.held.insert("release/held".into());
    assert!(!inbox.holds(&key));
    assert!(inbox.holds(&Key::parse("release/held").unwrap()));
    let files = [RawFile {
        path: "notes.md".into(),
        bytes: b"notes".to_vec(),
    }];
    assert_eq!(
        bank.bank(&key, &files, None),
        Ok("/data/inbox/release/x".into())
    );
    let taken = Taken {
        capture: Capture {
            key: key.clone(),
            source: "release".into(),
            event_at: "2026-10-10T14:02:11Z".into(),
            title: None,
            url: None,
            pointer: Some("deneb:/data/inbox/release/x".into()),
            came_in: CameIn::Intake,
            machine: "deneb".into(),
            taken_at: "2026-10-17T04:30:05Z".into(),
            unread: true,
        },
        own_signal: None,
    };
    inbox
        .refuse
        .insert("release/x".into(), "the tip moved".into());
    assert!(inbox.write(&taken).is_err());
    bank.discard(&key);
    assert!(bank.banked.is_empty());
    assert_eq!(
        *calls.borrow(),
        [
            Call::Holds("release/x".into()),
            Call::Holds("release/held".into()),
            Call::Bank("release/x".into()),
            Call::Write("release/x".into()),
            Call::Discard("release/x".into()),
        ]
    );
}

/// An offer line of the release `key`, with `extra` fields after the three
/// it needs.
fn offer(key: &str, extra: &str) -> String {
    format!(
        r#"{{"capture": {{"key": "{key}", "source": "release", "event_at": "2026-10-10T14:02:11Z"{extra}}}}}"#
    )
}

const RAW_AND_EXCERPT: &str = r#", "raw": ["notes.md"], "excerpt": {"text": "the notes", "kind": "ask", "asserted_by": "jdx/mise", "assertion": "a release", "subjects": [], "argues_with": [], "position": "notes"}"#;

fn keys(report: &weir_capture::Report) -> Vec<&str> {
    report.taken.iter().map(Key::as_str).collect()
}

#[test]
fn one_bad_key_among_fourteen() {
    let lines: Vec<String> = (1..=14)
        .map(|n| {
            if n == 5 {
                offer("moves/x", "")
            } else {
                offer(&format!("release/tool/{n}"), "")
            }
        })
        .collect();
    let (mut inbox, mut bank, _) = host();
    let report = take(&lines, None, &take_on("deneb"), &mut inbox, &mut bank);
    assert_eq!(report.taken.len(), 13);
    assert_eq!(inbox.written.len(), 13);
    assert!(!keys(&report).contains(&"moves/x"));
    assert_eq!(report.refused.len(), 1);
    let Refused { line, key, reason } = &report.refused[0];
    assert_eq!(*line, 5);
    assert_eq!(key.as_deref(), Some("moves/x"));
    assert!(reason.contains("`moves`"), "{reason}");
}

#[test]
fn a_line_that_is_not_json_is_named_and_passed() {
    let lines = [
        offer("release/a/1", ""),
        "not json".into(),
        offer("release/b/1", ""),
    ];
    let (mut inbox, mut bank, _) = host();
    let report = take(&lines, None, &take_on("deneb"), &mut inbox, &mut bank);
    assert_eq!(keys(&report), ["release/a/1", "release/b/1"]);
    assert_eq!(report.refused.len(), 1);
    assert_eq!(report.refused[0].line, 2);
    assert_eq!(report.refused[0].key, None);
    assert!(report.refused[0].reason.contains("not JSON"));
}

#[test]
fn the_same_release_the_next_day() {
    let (mut inbox, mut bank, calls) = host();
    inbox.held.insert("release/mise/2026.10.10".into());
    let lines = [offer("release/mise/2026.10.10", RAW_AND_EXCERPT)];
    let report = take(&lines, None, &take_on("deneb"), &mut inbox, &mut bank);
    assert_eq!(report.held, 1);
    assert!(report.taken.is_empty());
    assert!(report.refused.is_empty());
    assert!(inbox.written.is_empty());
    assert!(bank.banked.is_empty());
    assert_eq!(
        *calls.borrow(),
        [Call::Holds("release/mise/2026.10.10".into())]
    );
}

#[test]
fn one_key_twice_in_one_run() {
    let (mut inbox, mut bank, calls) = host();
    let lines = [
        offer("deploy/a/1", ""),
        offer("deploy/a/1", r#", "title": "again""#),
    ];
    let report = take(&lines, None, &take_on("deneb"), &mut inbox, &mut bank);
    assert_eq!(keys(&report), ["deploy/a/1"]);
    assert_eq!(report.held, 1);
    assert_eq!(inbox.written.len(), 1);
    assert_eq!(inbox.written[0].capture.title, None);
    assert_eq!(
        *calls.borrow(),
        [
            Call::Holds("deploy/a/1".into()),
            Call::Write("deploy/a/1".into())
        ]
    );
}
