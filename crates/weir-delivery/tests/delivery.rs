//! A delivery's records, held to delivery-records' "A delivery's records are
//! the four of C.8", "A message keeps its lines" and "A delivery's expiry is
//! its recorded time and its span" (100, 104; C.8; core.49).

use weir_delivery::{
    Confirmation, Delivery, Handover, Key, Known, Message, Reader, Record, Span, Time, Way,
};

fn key() -> Key {
    Key::parse("place/7f3a/end/2").unwrap()
}

fn reader(name: &str) -> Reader {
    Reader::parse(name).unwrap()
}

fn at(text: &str) -> Time {
    Time::parse(text, "at").unwrap()
}

fn delivery(message: &str, readers: &[&str], recorded_at: &str, span: u64) -> Delivery {
    Delivery::new(
        key(),
        Message::parse(message).unwrap(),
        readers.iter().map(|name| reader(name)).collect(),
        at(recorded_at),
        Span::seconds(span).unwrap(),
    )
    .unwrap()
}

#[test]
fn each_record_names_its_format() {
    let records: [Record; 4] = [
        delivery(
            "construct ended: the change is written.",
            &["planner"],
            "2026-11-08T10:02:00-05:00",
            604800,
        )
        .into(),
        Handover {
            key: key(),
            reader: reader("planner"),
            by: Way::parse("notice").unwrap(),
            at: at("2026-11-08T10:02:05-05:00"),
        }
        .into(),
        Confirmation {
            key: key(),
            reader: reader("planner"),
            at: at("2026-11-08T10:40:12-05:00"),
        }
        .into(),
        Known {
            reader: reader("planner"),
            known_at: at("2026-11-08T10:40:12-05:00"),
        }
        .into(),
    ];
    assert_eq!(
        records.each_ref().map(Record::format),
        [
            "weir-delivery/1",
            "weir-handover/1",
            "weir-confirmation/1",
            "weir-known/1"
        ]
    );
}

#[test]
fn a_weeks_span() {
    let delivery = delivery(
        "construct ended.",
        &["planner", "reviewer"],
        "2026-11-08T10:02:00-05:00",
        604800,
    );
    assert_eq!(delivery.recorded_at().as_str(), "2026-11-08T10:02:00-05:00");
    assert_eq!(delivery.expires_at().as_str(), "2026-11-15T10:02:00-05:00");
    let readers: Vec<&str> = delivery.readers().iter().map(Reader::as_str).collect();
    assert_eq!(readers, ["planner", "reviewer"]);
    let utc = self::delivery("x", &["planner"], "2026-11-08T23:59:30Z", 45);
    assert_eq!(utc.expires_at().as_str(), "2026-11-09T00:00:15Z");
}

#[test]
fn a_message_of_several_lines() {
    let lines = "construct ended: the change is written.\n\nRead its tasks next.";
    let delivery = delivery(lines, &["planner"], "2026-11-08T10:02:00-05:00", 60);
    assert_eq!(delivery.message().as_str(), lines);
    assert_eq!(delivery.message().as_str().split('\n').count(), 3);
}

#[test]
fn a_delivery_out_of_its_shape() {
    let refused = |readers: Vec<Reader>| {
        Delivery::new(
            key(),
            Message::parse("x").unwrap(),
            readers,
            at("2026-11-08T10:02:00-05:00"),
            Span::seconds(60).unwrap(),
        )
        .unwrap_err()
    };
    let none = refused(vec![]);
    assert_eq!(
        (none.field.as_str(), none.reason.as_str()),
        ("readers", "names no reader")
    );
    let twice = refused(vec![reader("planner"), reader("planner")]);
    assert_eq!(twice.field, "readers");
    assert!(twice.reason.contains("`planner` twice"), "{twice}");
    let past = Delivery::new(
        key(),
        Message::parse("x").unwrap(),
        vec![reader("planner")],
        at("9999-12-31T23:59:59Z"),
        Span::seconds(1).unwrap(),
    )
    .unwrap_err();
    assert_eq!(past.field, "span");
}
