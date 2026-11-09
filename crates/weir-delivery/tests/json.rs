//! A delivery's records in JSON, held to delivery-records' "The records are
//! spelled in JSON" (C.8; core.52).

use weir_delivery::{json, Reader, Record};

/// C.8's example, its four lines as written at blueprints 006fc34.
const C8: &str = r#"{"format": "weir-delivery/1", "key": "place/7f3a/end/2", "readers": ["planner"], "message": "construct ended: the change is written.", "recorded_at": "2026-11-08T10:02:00-05:00", "expires_at": "2026-11-15T10:02:00-05:00"}
{"format": "weir-handover/1", "key": "place/7f3a/end/2", "reader": "planner", "by": "notice", "at": "2026-11-08T10:02:05-05:00"}
{"format": "weir-confirmation/1", "key": "place/7f3a/end/2", "reader": "planner", "at": "2026-11-08T10:40:12-05:00"}
{"format": "weir-known/1", "reader": "planner", "known_at": "2026-11-08T10:40:12-05:00"}
"#;

#[test]
fn c8s_example_read() {
    let records = json::read_lines(C8).unwrap();
    let [Record::Delivery(delivery), Record::Handover(handover), Record::Confirmation(confirmation), Record::Known(known)] =
        records.as_slice()
    else {
        panic!("{records:?}");
    };
    assert_eq!(delivery.key().as_str(), "place/7f3a/end/2");
    assert_eq!(delivery.readers(), [Reader::parse("planner").unwrap()]);
    assert_eq!(
        delivery.message().as_str(),
        "construct ended: the change is written."
    );
    assert_eq!(delivery.expires_at().as_str(), "2026-11-15T10:02:00-05:00");
    assert_eq!(
        (handover.reader.as_str(), handover.by.as_str()),
        ("planner", "notice")
    );
    assert_eq!(confirmation.at.as_str(), "2026-11-08T10:40:12-05:00");
    assert_eq!(
        (known.reader.as_str(), known.known_at.as_str()),
        ("planner", "2026-11-08T10:40:12-05:00")
    );
    assert_eq!(
        json::write_lines(&records),
        C8,
        "spelled back byte for byte"
    );
    for (line, record) in C8.lines().zip(&records) {
        assert_eq!(json::write(record), line);
        assert_eq!(&json::read(line).unwrap(), record);
    }
}

#[test]
fn a_delivery_with_its_readers_left_out() {
    let line = r#"{"format": "weir-delivery/1", "key": "place/7f3a/end/2", "message": "m", "recorded_at": "2026-11-08T10:02:00-05:00", "expires_at": "2026-11-15T10:02:00-05:00"}"#;
    let refused = json::read(line).unwrap_err();
    assert_eq!(refused.field, "readers");
    let none = line.replace(r#""message""#, r#""readers": [], "message""#);
    assert_eq!(json::read(&none).unwrap_err().field, "readers");
}

#[test]
fn a_line_out_of_its_format() {
    let confirmation = r#"{"format": "weir-confirmation/1", "key": "k", "reader": "planner", "at": "2026-11-08T10:40:12-05:00"}"#;
    for (line, field) in [
        (
            confirmation.replace(r#""at""#, r#""note": "x", "at""#),
            "note",
        ),
        (
            confirmation.replace(r#""reader": "planner", "#, ""),
            "reader",
        ),
        (
            confirmation.replace(r#""reader": "planner""#, r#""reader": 7"#),
            "reader",
        ),
        (
            confirmation.replace(r#""at""#, r#""reader": "reviewer", "at""#),
            "reader",
        ),
        (
            confirmation.replace("confirmation/1", "confirmation/2"),
            "format",
        ),
        (confirmation.replace("10:40:12-05:00", "10:40:12"), "at"),
        ("[]".to_string(), "line"),
    ] {
        let refused = json::read(&line).unwrap_err();
        assert_eq!(refused.field, field, "{line}: {refused}");
    }
    let refused = json::read_lines(&format!("{confirmation}\n\n{{}}\n")).unwrap_err();
    assert_eq!(refused.field, "format");
    assert!(refused.reason.ends_with("on line 3"), "{refused}");
}
