//! A delivery's values, held to delivery-records' "A key, a reader and a
//! way are each one line" and "A time is RFC 3339, kept as written" (C.8;
//! core.14, core.47).

use weir_delivery::{Key, Message, Reader, Span, Time, Way};

#[test]
fn a_key_holding_a_line_break() {
    let refused = Key::parse("place/7f3a\nend/2").unwrap_err();
    assert_eq!(refused.field, "key");
    assert!(refused.reason.contains("a key is one line"), "{refused}");
    let carriage = Key::parse("place/7f3a\rend/2").unwrap_err();
    assert_eq!(carriage.field, "key");
    assert_eq!(
        Key::parse("place/7f3a/end/2").unwrap().as_str(),
        "place/7f3a/end/2"
    );
}

#[test]
fn a_reader_with_a_space_at_its_end() {
    let refused = Reader::parse("planner ").unwrap_err();
    assert_eq!(refused.field, "reader");
    for text in [" planner", "\tplanner", "planner\t", ""] {
        assert_eq!(Reader::parse(text).unwrap_err().field, "reader", "{text:?}");
    }
    assert_eq!(Reader::parse("plan ner").unwrap().as_str(), "plan ner");
    assert_eq!(Way::parse("notice ").unwrap_err().field, "by");
    assert_eq!(
        Way::parse("session-start").unwrap().as_str(),
        "session-start"
    );
}

#[test]
fn a_time_with_no_offset() {
    let refused = Time::parse("2026-11-08T10:40:12", "at").unwrap_err();
    assert_eq!(refused.field, "at");
    assert!(refused.reason.contains("2026-11-08T10:40:12"), "{refused}");
    for text in ["", "yesterday", "2026-11-08", "2026-11-08T10:40:12-05:00 "] {
        assert_eq!(
            Time::parse(text, "now").unwrap_err().field,
            "now",
            "{text:?}"
        );
    }
}

#[test]
fn a_time_is_kept_as_written() {
    let east = Time::parse("2026-11-08T10:40:12-05:00", "at").unwrap();
    let utc = Time::parse("2026-11-08T15:40:12Z", "at").unwrap();
    assert_eq!(east.as_str(), "2026-11-08T10:40:12-05:00");
    assert_eq!(east.instant(), utc.instant());
    assert_ne!(east, utc, "two spellings of one instant are two times");
    let later = Time::parse("2026-11-08T10:40:13-05:00", "at").unwrap();
    assert!(east < later && utc < later);
}

#[test]
fn a_message_and_a_span() {
    assert_eq!(Message::parse("").unwrap_err().field, "message");
    let lines = "construct ended.\n\nthe change is written.";
    assert_eq!(Message::parse(lines).unwrap().as_str(), lines);
    let refused = Span::seconds(0).unwrap_err();
    assert_eq!(
        (refused.field.as_str(), refused.reason.as_str()),
        ("span", "is 0 seconds")
    );
    assert_eq!(Span::seconds(604800).unwrap().as_seconds(), 604800);
}
