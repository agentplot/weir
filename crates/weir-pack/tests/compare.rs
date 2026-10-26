//! The comparisons of a host case, over made-up reports and tips, held to
//! the scenarios of "The report a host prints is compared with the step's"
//! and "The records at the tip are compared as written" that the
//! comparison alone decides (host-conformance; 15, 17, 18, 21 to 24; S4,
//! S15; C.7).

use std::collections::BTreeMap;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{Failure, Key, Refused, Report};
use weir_pack::{
    compare_report, compare_tip, read_host_case, verdict, HostCase, Machine, Pack, Shape,
};

const NOTES: &str = "## acme 1.0\n\n- the shim fixed\n";

fn lines() -> String {
    format!(
        "{}\n{}\n",
        serde_json::json!({"capture": {"key": "release/acme/1.0", "source": "release", "event_at": "2026-10-10T14:02:11Z", "raw": ["acme/notes.md"]}}),
        r#"{"done": {"listed": 1, "failed": []}}"#
    )
}

/// tip/'s capture: its three fields of the host's own written as their
/// form.
const WANTED: &str = "format: weir-capture/1
key: release/acme/1.0
source: release
event_at: 2026-10-10T14:02:11Z
pointer: <this machine>:<the bank's path>
came_in: intake
machine: <this machine>
taken_at: <when it was taken>
unread: yes
";

const CAPTURE: &str = "inbox/captures/release/acme/1.0/capture.rec";

fn case() -> HostCase {
    let lines = lines();
    let files = [
        (
            "case.toml",
            "satisfies = [17, 18, 22]\nlimit = 8192\n\n[[step]]\nintake = \"lines.jsonl\"\nraw = \"raw\"\ntaken = [\"release/acme/1.0\"]\nheld = 0\n\n[[step.refused]]\nline = 3\nkey = \"moves/x\"\nreason_holds = \"first segment\"\n",
        ),
        ("lines.jsonl", lines.as_str()),
        ("raw/acme/notes.md", NOTES),
        ("tip/inbox/captures/release/acme/1.0/capture.rec", WANTED),
    ];
    let pack = Pack::from_files(
        files
            .iter()
            .map(|(path, text)| (format!("host/made/{path}"), text.as_bytes().to_vec())),
    );
    read_host_case(&pack.case(Shape::Host, "made").unwrap()).unwrap()
}

fn at(text: &str) -> OffsetDateTime {
    OffsetDateTime::parse(text, &Rfc3339).unwrap()
}

/// The capture as a host on `deneb` writes it at `taken_at`, its raw
/// material banked at `/bank/release/acme/1.0`.
fn written(taken_at: &str) -> String {
    WANTED
        .replace(
            "<this machine>:<the bank's path>",
            "deneb:/bank/release/acme/1.0",
        )
        .replace("<this machine>", "deneb")
        .replace("<when it was taken>", taken_at)
}

/// Compares `tip` with the case on `deneb`, the case running from 12:00 to
/// 12:01, the bank holding `banked` at its one path.
fn compare(tip: &[(&str, &str)], banked: &[&str]) -> Vec<String> {
    let tip: BTreeMap<String, Vec<u8>> = tip
        .iter()
        .map(|(path, text)| (path.to_string(), text.as_bytes().to_vec()))
        .collect();
    let banked: Vec<Vec<u8>> = banked.iter().map(|text| text.as_bytes().to_vec()).collect();
    let read = move |path: &str| {
        if path == "/bank/release/acme/1.0" {
            Ok(banked.clone())
        } else {
            Err(format!("no folder {path}"))
        }
    };
    compare_tip(
        &case(),
        &tip,
        &Machine {
            hostname: "deneb",
            began: at("2026-10-26T12:00:00.250Z"),
            ended: at("2026-10-26T12:01:00.500Z"),
            read: &read,
        },
    )
}

fn report(refusal_reason: &str) -> Report {
    Report {
        taken: vec![Key::parse("release/acme/1.0").unwrap()],
        refused: vec![Refused {
            line: 3,
            key: Some("moves/x".into()),
            reason: refusal_reason.into(),
        }],
        ..Report::default()
    }
}

#[test]
fn a_report_that_agrees() {
    let case = case();
    let printed = report("`key` is `moves/x`: a key's first segment is never `moves`").to_json();
    assert_eq!(
        compare_report(&case.steps[0].expected, &printed),
        Vec::<String>::new()
    );
}

#[test]
fn a_refusal_with_another_reason() {
    let case = case();
    let said = compare_report(&case.steps[0].expected, &report("bad key").to_json());
    assert_eq!(
        said,
        ["it refused line 3 for `moves/x` because `bad key`, and the case expects a reason holding `first segment`"]
    );
}

#[test]
fn what_a_report_took_held_and_refused_otherwise() {
    let case = case();
    let mut other = report("first segment");
    other.taken.clear();
    other.held = 1;
    other.refused[0].line = 4;
    let said = compare_report(&case.steps[0].expected, &other.to_json());
    assert_eq!(said.len(), 4, "{said:?}");
    assert!(said[0].starts_with("it took []"), "{}", said[0]);
    assert!(said[1].starts_with("it held 1"), "{}", said[1]);
    assert!(
        said[2].starts_with("it did not refuse line 3 for `moves/x`"),
        "{}",
        said[2]
    );
    assert!(said[3].contains("line 4 for `moves/x`"), "{}", said[3]);
    assert!(
        said[3].ends_with("which the case does not expect"),
        "{}",
        said[3]
    );
}

#[test]
fn a_run_whose_cursor_is_kept_from_an_incomplete_run() {
    let pack = Pack::from_files(
        [
            (
                "case.toml",
                "satisfies = [9]\nlimit = 8192\n\n[[step]]\nrun = \"adapter.sh\"\nlines = \"lines.jsonl\"\ntaken = [\"release/acme/1.0\"]\nheld = 0\ncomplete = false\n",
            ),
            ("adapter.sh", "#!/bin/sh\n"),
            ("lines.jsonl", "\n"),
        ]
        .iter()
        .map(|(path, text)| (format!("host/run/{path}"), text.as_bytes().to_vec())),
    );
    let case = read_host_case(&pack.case(Shape::Host, "run").unwrap()).unwrap();
    let kept = Report {
        taken: vec![Key::parse("release/acme/1.0").unwrap()],
        last_cursor: Some("c5".into()),
        cursor: Some("c5".into()),
        ..Report::default()
    };
    assert_eq!(
        compare_report(&case.steps[0].expected, &kept.to_json()),
        ["it kept the cursor `c5`, and the case expects none"]
    );
    let complete = Report {
        listed: Some(5),
        failed: vec![Failure {
            what: "x".into(),
            why: "y".into(),
        }],
        complete: true,
        ..kept
    };
    let said = compare_report(&case.steps[0].expected, &complete.to_json());
    assert_eq!(said.len(), 4, "{said:?}");
    assert!(said[0].starts_with("it reports the listing's count as `5`, and the case expects none"));
    assert!(said[3].starts_with("it reports the run complete, and the case expects it incomplete"));
}

#[test]
fn no_report() {
    let case = case();
    let said = compare_report(&case.steps[0].expected, "taken: release/acme/1.0");
    assert_eq!(said.len(), 1);
    assert!(
        said[0].starts_with("the report does not read as weir-report/1"),
        "{}",
        said[0]
    );
}

#[test]
fn a_tip_that_agrees() {
    assert_eq!(
        compare(&[(CAPTURE, &written("2026-10-26T12:00:30Z"))], &[NOTES]),
        Vec::<String>::new()
    );
}

#[test]
fn a_field_of_the_hosts_own() {
    // Two hosts that keep the contract, at different times.
    for taken_at in [
        "2026-10-26T12:00:00Z",
        "2026-10-26T12:00:59.999Z",
        "2026-10-26T14:00:30+02:00",
    ] {
        assert_eq!(
            compare(&[(CAPTURE, &written(taken_at))], &[NOTES]),
            Vec::<String>::new()
        );
    }
    let late = compare(&[(CAPTURE, &written("2026-10-26T12:05:00Z"))], &[NOTES]);
    assert_eq!(late.len(), 1);
    assert!(
        late[0]
            .contains("`taken_at` is `2026-10-26T12:05:00Z`, not a time within the case's running"),
        "{}",
        late[0]
    );
    let elsewhere = written("2026-10-26T12:00:30Z").replace("machine: deneb", "machine: vega");
    let said = compare(&[(CAPTURE, &elsewhere)], &[NOTES]);
    assert_eq!(said.len(), 2, "{said:?}");
    assert!(
        said[0].contains("names the machine `deneb`, and the record's machine is `vega`"),
        "{}",
        said[0]
    );
    assert!(
        said[1].contains("`machine` is `vega`, and this machine is `deneb`"),
        "{}",
        said[1]
    );
}

#[test]
fn a_host_keeps_raw_material_in_a_record() {
    let kept =
        written("2026-10-26T12:00:30Z").replace("unread: yes\n", "unread: yes\nraw: ## acme 1.0\n");
    let said = compare(&[(CAPTURE, &kept)], &[NOTES]);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with(&format!("{CAPTURE} differs from tip/'s")),
        "{}",
        said[0]
    );
    let case = case();
    assert!(verdict(&case, &said).starts_with("made: failed, citing 17, 18, 22: "));
    assert_eq!(verdict(&case, &[]), "made: passed");
}

#[test]
fn a_host_keeps_raw_material_beside_its_record() {
    let said = compare(
        &[
            (CAPTURE, &written("2026-10-26T12:00:30Z")),
            ("inbox/captures/release/acme/1.0/notes.md", NOTES),
        ],
        &[NOTES],
    );
    assert_eq!(
        said,
        ["the tip holds inbox/captures/release/acme/1.0/notes.md, which tip/ does not"]
    );
    let lacks = compare(&[], &[NOTES]);
    assert_eq!(
        lacks,
        [format!("the tip lacks {CAPTURE}, which tip/ holds")]
    );
}

#[test]
fn a_pointer_to_a_bank_that_lacks_a_file() {
    let said = compare(
        &[(CAPTURE, &written("2026-10-26T12:00:30Z"))],
        &["other bytes"],
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].starts_with(CAPTURE), "{}", said[0]);
    assert!(
        said[0].contains("does not hold the raw `acme/notes.md`"),
        "{}",
        said[0]
    );
    let nowhere = written("2026-10-26T12:00:30Z").replace("/bank/", "/elsewhere/");
    let said = compare(&[(CAPTURE, &nowhere)], &[NOTES]);
    assert!(said[0].contains("cannot be read here"), "{}", said[0]);
    let no_pointer =
        written("2026-10-26T12:00:30Z").replace("pointer: deneb:/bank/release/acme/1.0\n", "");
    let said = compare(&[(CAPTURE, &no_pointer)], &[NOTES]);
    assert!(
        said[0].contains("differs from tip/'s at line 5"),
        "{}",
        said[0]
    );
}
