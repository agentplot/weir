//! Taking lines through a host's inbox and bank, held to the scenarios of
//! capture-take, with in-memory fakes of both.

mod fakes;

use std::fs;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{take, Bank, Inbox, RawFile, Refused, Take, TakeError, Taken};
use weir_capture::{CameIn, Capture, Key, SMALLEST_LIMIT};

use fakes::{host, Banked, Call};

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

fn raw_place(files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let place = tempfile::tempdir().unwrap();
    for (path, bytes) in files {
        let path = place.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    place
}

fn raw(paths: &[&str]) -> String {
    let paths: Vec<String> = paths.iter().map(|path| format!("\"{path}\"")).collect();
    format!(r#", "raw": [{}]"#, paths.join(", "))
}

/// An own excerpt of `text`, which must need no JSON escape but newlines.
fn excerpt(text: &str) -> String {
    format!(
        r#", "excerpt": {{"text": "{}", "kind": "ask", "asserted_by": "jdx/mise", "assertion": "mise is released", "subjects": ["release", "mise"], "argues_with": [], "position": "release notes"}}"#,
        text.replace('\n', "\\n")
    )
}

const NOTES: &[u8] = b"## mise 2026.10.10\n\n- the shim\n";
const RELEASE: &[u8] = &[0x7b, 0x00, 0xff, 0xfe, 0x7d, 0x0a];

#[test]
fn the_bank_is_filled_before_the_inbox() {
    let place = raw_place(&[("m/notes.md", NOTES), ("m/release.json", RELEASE)]);
    let (mut inbox, mut bank, calls) = host();
    let lines = [offer(
        "release/mise/2026.10.10",
        &raw(&["m/notes.md", "m/release.json"]),
    )];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert_eq!(keys(&report), ["release/mise/2026.10.10"]);
    assert_eq!(
        bank.banked["release/mise/2026.10.10"],
        Banked {
            files: vec![
                RawFile {
                    path: "m/notes.md".into(),
                    bytes: NOTES.to_vec()
                },
                RawFile {
                    path: "m/release.json".into(),
                    bytes: RELEASE.to_vec()
                },
            ],
            whole_excerpt: None,
        }
    );
    assert_eq!(
        *calls.borrow(),
        [
            Call::Holds("release/mise/2026.10.10".into()),
            Call::Bank("release/mise/2026.10.10".into()),
            Call::Write("release/mise/2026.10.10".into()),
        ]
    );
}

#[test]
fn a_raw_file_that_is_not_there() {
    let place = raw_place(&[("other.md", b"x")]);
    let (mut inbox, mut bank, calls) = host();
    let lines = [offer("release/a/1", &raw(&["notes.md"]))];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert!(report.taken.is_empty());
    assert_eq!(report.refused.len(), 1);
    assert!(
        report.refused[0].reason.contains("`notes.md`"),
        "{}",
        report.refused[0].reason
    );
    assert!(bank.banked.is_empty());
    assert!(inbox.written.is_empty());
    assert_eq!(*calls.borrow(), [Call::Holds("release/a/1".into())]);
}

#[test]
fn intake_with_no_raw_place() {
    let (mut inbox, mut bank, _) = host();
    let intake = Take::new("deneb", at("2026-10-17T04:30:05Z"), LIMIT, CameIn::Intake).unwrap();
    let lines = [
        offer("deploy/a/1", &raw(&["deploy.log"])),
        offer("deploy/a/2", ""),
    ];
    let report = take(&lines, None, &intake, &mut inbox, &mut bank);
    assert_eq!(keys(&report), ["deploy/a/2"]);
    assert_eq!(report.refused[0].line, 1);
    let reason = &report.refused[0].reason;
    assert!(reason.contains("`deploy.log`"), "{reason}");
    assert!(reason.contains("no raw place"), "{reason}");
    assert!(bank.banked.is_empty());
}

#[cfg(unix)]
#[test]
fn a_raw_file_that_resolves_outside_the_raw_place() {
    let outside = raw_place(&[("secret.md", b"not the adapter's")]);
    let place = raw_place(&[]);
    std::os::unix::fs::symlink(
        outside.path().join("secret.md"),
        place.path().join("notes.md"),
    )
    .unwrap();
    let (mut inbox, mut bank, _) = host();
    let lines = [offer("release/a/1", &raw(&["notes.md"]))];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert!(report.taken.is_empty());
    let reason = &report.refused[0].reason;
    assert!(reason.contains("outside the raw place"), "{reason}");
    assert!(bank.banked.is_empty());
}

#[test]
fn a_banked_capture_points_at_its_machine() {
    let place = raw_place(&[("notes.md", NOTES)]);
    let (mut inbox, mut bank, _) = host();
    let lines = [offer("release/mise/2026.10.10", &raw(&["notes.md"]))];
    take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert_eq!(
        inbox.written[0].capture.pointer.as_deref(),
        Some("deneb:/data/inbox/release/mise/2026.10.10")
    );
}

#[test]
fn nothing_banked_no_pointer() {
    let (mut inbox, mut bank, calls) = host();
    let lines = [
        offer("release/a/1", ""),
        offer("release/a/2", &excerpt("a short excerpt, kept whole")),
    ];
    take(&lines, None, &take_on("deneb"), &mut inbox, &mut bank);
    assert_eq!(inbox.written.len(), 2);
    assert!(inbox.written.iter().all(|t| t.capture.pointer.is_none()));
    assert!(!calls
        .borrow()
        .iter()
        .any(|call| matches!(call, Call::Bank(_))));
}

/// Forty kibibytes of short lines.
fn forty_kibibytes() -> String {
    let mut text = String::new();
    let mut n = 0;
    while text.len() < 40 * 1024 {
        text.push_str(&format!("- change {n} of this release\n"));
        n += 1;
    }
    text
}

#[test]
fn notes_and_excerpt_both_banked() {
    let place = raw_place(&[("notes.md", NOTES)]);
    let (mut inbox, mut bank, calls) = host();
    let text = forty_kibibytes();
    let lines = [offer(
        "release/mise/2026.10.10",
        &format!("{}{}", raw(&["notes.md"]), excerpt(&text)),
    )];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert_eq!(keys(&report), ["release/mise/2026.10.10"]);
    let banked = &bank.banked["release/mise/2026.10.10"];
    assert_eq!(banked.files[0].bytes, NOTES);
    assert_eq!(banked.whole_excerpt.as_deref(), Some(text.as_str()));
    assert_eq!(inbox.written.len(), 1);
    assert_eq!(
        inbox.written[0].capture.pointer.as_deref(),
        Some("deneb:/data/inbox/release/mise/2026.10.10")
    );
    let order: Vec<Call> = calls.borrow().clone();
    assert_eq!(
        order[1..],
        [
            Call::Bank("release/mise/2026.10.10".into()),
            Call::Write("release/mise/2026.10.10".into())
        ]
    );
}

#[test]
fn a_bank_that_refuses_writes_nothing() {
    let place = raw_place(&[("notes.md", NOTES)]);
    let (mut inbox, mut bank, calls) = host();
    bank.refuse = Some("the disk is full".into());
    let lines = [offer("release/a/1", &raw(&["notes.md"]))];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert!(report.taken.is_empty());
    assert!(report.refused[0].reason.contains("the disk is full"));
    assert!(inbox.written.is_empty());
    assert_eq!(
        *calls.borrow(),
        [
            Call::Holds("release/a/1".into()),
            Call::Bank("release/a/1".into()),
            Call::Discard("release/a/1".into()),
        ]
    );
}

#[test]
fn a_raw_place_path_is_relative_to_the_place() {
    let place = raw_place(&[("deep/er/notes.md", NOTES)]);
    let (mut inbox, mut bank, _) = host();
    let lines = [offer("release/a/1", &raw(&["deep/er/notes.md"]))];
    let report = take(
        &lines,
        Some(place.path()),
        &take_on("deneb"),
        &mut inbox,
        &mut bank,
    );
    assert_eq!(keys(&report), ["release/a/1"]);
    assert_eq!(bank.banked["release/a/1"].files[0].path, "deep/er/notes.md");
}
