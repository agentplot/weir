//! The meeting checks, held to "The meeting checks hold a meeting adapter
//! to 66" (adapter-checks).

use std::collections::BTreeMap;

use weir_pack::{check_meeting_lines, Breach, Meetings, RawPlace};

fn meeting(key: &str, event_at: &str, more: &str) -> String {
    format!(
        r#"{{"capture": {{"key": "{key}", "source": "meeting", "event_at": "{event_at}", "raw": ["{key}/transcript.json", "{key}/transcript.txt"]{more}}}}}"#
    )
}

fn done(listed: u64) -> String {
    format!(r#"{{"done": {{"listed": {listed}, "failed": []}}}}"#)
}

fn raw() -> BTreeMap<String, Vec<u8>> {
    ["m-1", "m-2"]
        .iter()
        .flat_map(|id| {
            [
                (
                    format!("meeting/wispr/{id}/transcript.json"),
                    b"{}".to_vec(),
                ),
                (
                    format!("meeting/wispr/{id}/transcript.txt"),
                    b"words".to_vec(),
                ),
            ]
        })
        .collect()
}

/// The fixture: m-1 and m-2 with their starts, and `short` meetings too
/// short to hold anything.
fn fixture(short: u64) -> Meetings {
    Meetings {
        starts: [
            ("meeting/wispr/m-1", "2026-10-20T15:00:00-04:00"),
            ("meeting/wispr/m-2", "2026-10-21T09:30:00-04:00"),
        ]
        .into_iter()
        .map(|(key, start)| (key.to_string(), start.to_string()))
        .collect(),
        short,
    }
}

fn named(breaches: &[Breach]) -> Vec<(Option<usize>, u32)> {
    breaches.iter().map(|b| (b.line, b.clause)).collect()
}

#[test]
fn a_meeting_adapter_that_keeps_66() {
    let lines = [
        meeting("meeting/wispr/m-1", "2026-10-20T15:00:00-04:00", ""),
        // The same start, written in UTC.
        meeting("meeting/wispr/m-2", "2026-10-21T13:30:00Z", ""),
        r#"{"cursor": "m-2"}"#.to_string(),
        done(3),
    ];
    let raw = raw();
    assert_eq!(
        check_meeting_lines(&lines, RawPlace::Files(&raw), true, &fixture(1)),
        []
    );
}

#[test]
fn a_short_meeting_not_counted() {
    let lines = [
        meeting("meeting/wispr/m-1", "2026-10-20T15:00:00-04:00", ""),
        meeting("meeting/wispr/m-2", "2026-10-21T09:30:00-04:00", ""),
        done(2),
    ];
    let raw = raw();
    let breaches = check_meeting_lines(&lines, RawPlace::Files(&raw), true, &fixture(1));
    assert_eq!(named(&breaches), [(Some(3), 66)]);
    assert!(breaches[0].why.contains("3 together"), "{}", breaches[0]);
}

#[test]
fn an_event_time_that_is_not_the_start() {
    let lines = [
        meeting("meeting/wispr/m-1", "2026-10-20T16:12:40-04:00", ""),
        done(1),
    ];
    let raw = raw();
    let breaches = check_meeting_lines(&lines, RawPlace::Files(&raw), true, &fixture(0));
    assert_eq!(named(&breaches), [(Some(1), 66)]);
    assert!(
        breaches[0]
            .why
            .contains("started at `2026-10-20T15:00:00-04:00`"),
        "{}",
        breaches[0]
    );
}

#[test]
fn every_breach_of_66_and_of_the_contract() {
    let excerpt = r#", "excerpt": {"text": "words", "kind": "ask", "asserted_by": "a", "assertion": "b", "subjects": [], "argues_with": [], "position": "c"}"#;
    let lines = [
        meeting("meeting/m-1", "2026-10-20T15:00:00-04:00", ""),
        meeting("meeting/wispr/m-9", "2026-10-20T15:00:00-04:00", ""),
        meeting("meeting/wispr/m-1", "2026-10-20T15:00:00-04:00", "")
            .replace(r#""source": "meeting""#, r#""source": "wispr""#),
        r#"{"capture": {"key": "meeting/wispr/m-2", "source": "meeting", "event_at": "2026-10-21T09:30:00-04:00", "raw": ["meeting/wispr/m-2/transcript.txt"]}}"#.to_string(),
        // m-1 again, from the source line 3 did not give: another event
        // under its key.
        meeting("meeting/wispr/m-1", "2026-10-20T15:00:00-04:00", excerpt),
        "not json".to_string(),
    ];
    let raw = raw();
    let breaches = check_meeting_lines(&lines, RawPlace::Files(&raw), true, &fixture(0));
    assert_eq!(
        named(&breaches),
        [
            (Some(1), 6),
            (Some(1), 6),
            (Some(1), 66),
            (Some(2), 6),
            (Some(2), 6),
            (Some(2), 66),
            (Some(3), 66),
            (Some(4), 66),
            (Some(5), 3),
            (Some(5), 66),
            (Some(6), 1),
            (None, 9),
        ],
        "{breaches:#?}"
    );
}
