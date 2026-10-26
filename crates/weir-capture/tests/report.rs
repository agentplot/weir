//! The report a host prints last, held to the scenarios of "The report is
//! written and read as weir-report/1" (capture-take; 23, 76; C.7;
//! core.22).

// The fakes are shared with take.rs, which uses all of them.
#[allow(dead_code)]
mod fakes;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{read_report, take, Refused, Report, Take};
use weir_capture::{CameIn, Failure, Key, REPORT_FORMAT};

use fakes::host;

fn key(text: &str) -> Key {
    Key::parse(text).unwrap()
}

/// Two keys taken, one held, line 3 refused for `moves/x`, a listing of
/// fourteen with one failure, the cursor `c2` kept, and complete.
fn a_runs_report() -> Report {
    Report {
        taken: vec![key("release/acme/v1"), key("release/acme/v2")],
        held: 1,
        refused: vec![Refused {
            line: 3,
            key: Some("moves/x".into()),
            reason: "`key` is `moves/x`: a key's first segment is never `moves`, where an \
                     inbox's moves lie"
                .into(),
        }],
        listed: Some(14),
        failed: vec![Failure {
            what: "github:acme".into(),
            why: "HTTP 502".into(),
        }],
        last_cursor: Some("c2".into()),
        complete: true,
        cursor: Some("c2".into()),
    }
}

#[test]
fn a_runs_report_written_and_read_back() {
    let report = a_runs_report();
    let written = report.to_json();
    assert!(!written.contains('\n'), "{written}");
    assert_eq!(
        written,
        concat!(
            r#"{"format":"weir-report/1","taken":["release/acme/v1","release/acme/v2"],"#,
            r#""held":1,"refused":[{"line":3,"key":"moves/x","reason":"`key` is `moves/x`: "#,
            r#"a key's first segment is never `moves`, where an inbox's moves lie"}],"#,
            r#""listed":14,"failed":[{"what":"github:acme","why":"HTTP 502"}],"cursor":"c2","#,
            r#""complete":true}"#
        )
    );
    assert_eq!(read_report(&written), Ok(report));
}

#[test]
fn what_a_report_lacks_is_left_out() {
    let report = Report {
        taken: vec![key("deploy/a/1")],
        refused: vec![Refused {
            line: 2,
            key: None,
            reason: "the line is not JSON".into(),
        }],
        ..Report::default()
    };
    let written = report.to_json();
    assert_eq!(
        written,
        r#"{"format":"weir-report/1","taken":["deploy/a/1"],"held":0,"refused":[{"line":2,"reason":"the line is not JSON"}],"failed":[],"complete":false}"#
    );
    assert_eq!(read_report(&written), Ok(report));
}

#[test]
fn a_report_from_take_reads_back() {
    let (mut inbox, mut bank, _) = host();
    inbox.held.insert("release/held/1".into());
    let lines = [
        r#"{"capture": {"key": "release/a/1", "source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#,
        r#"{"capture": {"key": "release/held/1", "source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#,
        "not json",
        r#"{"cursor": "page=3&since=2026-10-17"}"#,
        r#"{"done": {"listed": 3, "failed": [{"what": "x", "why": "HTTP 502"}]}}"#,
    ];
    let how = Take::new(
        "deneb",
        OffsetDateTime::parse("2026-10-17T04:30:05Z", &Rfc3339).unwrap(),
        8192,
        CameIn::Run("releases".into()),
    )
    .unwrap();
    let report = take(lines, None, &how, &mut inbox, &mut bank).finish(true);
    assert!(report.complete);
    assert_eq!(read_report(&report.to_json()), Ok(report));
}

#[test]
fn another_format_is_refused_naming_it() {
    let written = a_runs_report()
        .to_json()
        .replace(REPORT_FORMAT, "weir-report/2");
    let refused = read_report(&written).unwrap_err();
    assert_eq!(refused.field.as_deref(), Some("format"));
    assert_eq!(
        refused.to_string(),
        "`format` is `weir-report/2`, not `weir-report/1`"
    );
}

#[test]
fn a_report_lacking_a_field_is_refused_naming_it() {
    let written: serde_json::Value = serde_json::from_str(&a_runs_report().to_json()).unwrap();
    for field in ["format", "taken", "held", "refused", "failed", "complete"] {
        let mut lacking = written.clone();
        lacking.as_object_mut().unwrap().remove(field);
        let refused = read_report(&lacking.to_string()).unwrap_err();
        assert_eq!(refused.field.as_deref(), Some(field), "{refused}");
        assert_eq!(refused.reason, "is missing");
    }
    let mut no_reason = written.clone();
    no_reason["refused"][0]
        .as_object_mut()
        .unwrap()
        .remove("reason");
    let refused = read_report(&no_reason.to_string()).unwrap_err();
    assert_eq!(refused.to_string(), "`refused[0].reason` is missing");
    let mut no_why = written;
    no_why["failed"][0].as_object_mut().unwrap().remove("why");
    let refused = read_report(&no_why.to_string()).unwrap_err();
    assert_eq!(refused.to_string(), "`failed[0].why` is missing");
}

#[test]
fn a_report_out_of_its_shape_is_refused() {
    for (text, field) in [
        (
            r#"{"format":"weir-report/1","taken":["moves/x"],"held":0,"refused":[],"failed":[],"complete":true}"#,
            "taken[0]",
        ),
        (
            r#"{"format":"weir-report/1","taken":[],"held":-1,"refused":[],"failed":[],"complete":true}"#,
            "held",
        ),
        (
            r#"{"format":"weir-report/1","taken":[],"held":0,"refused":[{"line":0,"reason":"x"}],"failed":[],"complete":true}"#,
            "refused[0].line",
        ),
        (
            r#"{"format":"weir-report/1","taken":[],"held":0,"refused":[],"failed":[],"complete":"yes"}"#,
            "complete",
        ),
        (
            r#"{"format":"weir-report/1","taken":[],"held":0,"refused":[],"failed":[],"cursor":3,"complete":true}"#,
            "cursor",
        ),
    ] {
        let refused = read_report(text).unwrap_err();
        assert_eq!(refused.field.as_deref(), Some(field), "{refused}");
    }
    assert_eq!(read_report("[]").unwrap_err().field, None);
    assert!(read_report("not json")
        .unwrap_err()
        .reason
        .starts_with("the report is not JSON"));
}
