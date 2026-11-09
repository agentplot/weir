//! `record`, held to delivery-calls' "record returns the delivery to add",
//! "record holds a key once" and "record refuses a delivery out of its
//! shape" (100, 101, 104, 108; S24; core.47).

use weir_delivery::{record, Delivery, Reader, Record, Recorded};

const KEY: &str = "place/7f3a/end/2";
const AT: &str = "2026-11-08T10:02:00-05:00";
const WEEK: u64 = 604800;

fn added(recorded: Recorded) -> Vec<Record> {
    match recorded {
        Recorded::Add(records) => records,
        held => panic!("{held:?}"),
    }
}

fn the_delivery(records: &[Record]) -> &Delivery {
    match records {
        [Record::Delivery(delivery)] => delivery,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_delivery_recorded_for_two_readers() {
    let records = added(
        record(
            &[],
            KEY,
            "construct ended: the change is written.",
            &["planner", "reviewer"],
            AT,
            WEEK,
        )
        .unwrap(),
    );
    let delivery = the_delivery(&records);
    assert_eq!(records[0].format(), "weir-delivery/1");
    assert_eq!(delivery.key().as_str(), KEY);
    let readers: Vec<&str> = delivery.readers().iter().map(Reader::as_str).collect();
    assert_eq!(readers, ["planner", "reviewer"]);
    assert_eq!(delivery.recorded_at().as_str(), AT);
    assert_eq!(delivery.expires_at().as_str(), "2026-11-15T10:02:00-05:00");
}

#[test]
fn one_key_recorded_twice() {
    let first = added(record(&[], KEY, "the first message", &["planner"], AT, WEEK).unwrap());
    let again = record(
        &first,
        KEY,
        "another message",
        &["reviewer"],
        "2026-11-09T08:00:00-05:00",
        60,
    )
    .unwrap();
    let Recorded::Held(held) = again else {
        panic!("{again:?}");
    };
    assert_eq!(&held, the_delivery(&first));
    assert_eq!(held.message().as_str(), "the first message");
    let other = record(&first, "place/7f3a/end/3", "m", &["planner"], AT, WEEK).unwrap();
    assert!(
        matches!(other, Recorded::Add(_)),
        "another key is no held key"
    );
}

#[test]
fn a_delivery_with_no_reader() {
    let refused = record(&[], KEY, "m", &[], AT, WEEK).unwrap_err();
    assert_eq!(refused.field, "readers");
    assert!(refused.reason.contains("names no reader"), "{refused}");
}

#[test]
fn a_reader_named_twice() {
    let refused = record(&[], KEY, "m", &["planner", "planner"], AT, WEEK).unwrap_err();
    assert_eq!(refused.field, "readers");
    assert!(refused.reason.contains("planner"), "{refused}");
    assert!(refused.reason.contains("twice"), "{refused}");
}

#[test]
fn a_span_of_no_seconds() {
    let refused = record(&[], KEY, "m", &["planner"], AT, 0).unwrap_err();
    assert_eq!(refused.field, "span");
    assert!(refused.reason.contains("is 0 seconds"), "{refused}");
}

#[test]
fn a_call_refused_at_its_first_field() {
    for (key, message, readers, at, span, field) in [
        ("", "", &[][..], "", 0, "key"),
        (KEY, "", &[][..], "", 0, "message"),
        (KEY, "m", &["planner "][..], "", 0, "readers"),
        (KEY, "m", &[][..], "", 0, "readers"),
        (KEY, "m", &["planner"][..], "2026-11-08", 0, "at"),
        (KEY, "m", &["planner"][..], AT, 0, "span"),
    ] {
        let refused = record(&[], key, message, readers, at, span).unwrap_err();
        assert_eq!(refused.field, field, "{refused}");
    }
}
