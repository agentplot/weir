//! `owed`, held to delivery-calls' "The calls are functions over the
//! records their caller passes", "owed gives every delivery a reader has
//! not confirmed", "owed marks a delivery past its expiry expired", "owed
//! says whether a reader's name is known" and "owed lists the deliveries
//! done" (104 to 107, 111; I9; S22, S23; core.49, core.50, core.51).

use weir_delivery::{
    owed, Confirmation, Delivery, Handover, Key, Known, Message, Owed, Reader, Record, Span, Time,
    Way,
};

const AT: &str = "2026-11-08T10:02:00-05:00";

fn key(text: &str) -> Key {
    Key::parse(text).unwrap()
}

fn reader(name: &str) -> Reader {
    Reader::parse(name).unwrap()
}

fn at(text: &str) -> Time {
    Time::parse(text, "at").unwrap()
}

fn delivery(under: &str, readers: &[&str], span: u64) -> Record {
    Delivery::new(
        key(under),
        Message::parse("construct ended: the change is written.").unwrap(),
        readers.iter().map(|name| reader(name)).collect(),
        at(AT),
        Span::seconds(span).unwrap(),
    )
    .unwrap()
    .into()
}

fn handover(under: &str, to: &str, by: &str, when: &str) -> Record {
    Handover {
        key: key(under),
        reader: reader(to),
        by: Way::parse(by).unwrap(),
        at: at(when),
    }
    .into()
}

fn confirmation(under: &str, by: &str, when: &str) -> Record {
    Confirmation {
        key: key(under),
        reader: reader(by),
        at: at(when),
    }
    .into()
}

fn known(name: &str, when: &str) -> Record {
    Known {
        reader: reader(name),
        known_at: at(when),
    }
    .into()
}

/// Each reader owed, with the keys it is owed, as `owed` gives them.
fn keys(owed: &Owed) -> Vec<(&str, Vec<&str>)> {
    owed.readers
        .iter()
        .map(|owed| {
            (
                owed.reader.as_str(),
                owed.deliveries
                    .iter()
                    .map(|owed| owed.delivery.key().as_str())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn an_expired_delivery_for_a_misspelt_reader() {
    let records = [delivery("place/7f3a/end/2", &["planer"], 86400)];
    let owed = owed(&records, "2026-11-12T09:00:00-05:00").unwrap();
    let [planer] = owed.readers.as_slice() else {
        panic!("{owed:?}");
    };
    assert_eq!(planer.reader.as_str(), "planer");
    assert!(!planer.known, "no reader has confirmed under `planer`");
    let [delivery] = planer.deliveries.as_slice() else {
        panic!("{planer:?}");
    };
    assert!(delivery.expired);
    assert!(owed.done.is_empty());
    let before = weir_delivery::owed(&records, "2026-11-09T10:01:59-05:00").unwrap();
    assert!(!before.readers[0].deliveries[0].expired);
    let at_expiry = weir_delivery::owed(&records, "2026-11-09T15:02:00Z").unwrap();
    assert!(at_expiry.readers[0].deliveries[0].expired, "at its expiry");
}

#[test]
fn a_name_stays_known_after_its_deliveries_are_let_go_of() {
    let records = [
        known("planner", "2026-11-01T09:00:00-05:00"),
        delivery("place/7f3a/end/2", &["planner"], 604800),
    ];
    let owed = owed(&records, "2026-11-08T11:00:00-05:00").unwrap();
    assert_eq!(keys(&owed), [("planner", vec!["place/7f3a/end/2"])]);
    assert!(owed.readers[0].known);
}

#[test]
fn a_delivery_done_when_every_reader_confirms() {
    let records = [
        delivery("place/7f3a/end/2", &["planner", "reviewer"], 604800),
        handover(
            "place/7f3a/end/2",
            "planner",
            "notice",
            "2026-11-08T10:02:05-05:00",
        ),
        confirmation("place/7f3a/end/2", "planner", "2026-11-08T10:40:12-05:00"),
        known("planner", "2026-11-08T10:40:12-05:00"),
        handover(
            "place/7f3a/end/2",
            "reviewer",
            "session-start",
            "2026-11-08T11:00:00-05:00",
        ),
        confirmation("place/7f3a/end/2", "reviewer", "2026-11-08T11:00:00-05:00"),
        known("reviewer", "2026-11-08T11:00:00-05:00"),
    ];
    let owed = owed(&records, "2026-11-08T12:00:00-05:00").unwrap();
    assert!(owed.readers.is_empty(), "{owed:?}");
    assert_eq!(owed.done, [key("place/7f3a/end/2")]);
}

#[test]
fn a_delivery_one_reader_has_confirmed() {
    let records = [
        delivery("place/7f3a/end/2", &["planner", "reviewer"], 604800),
        handover(
            "place/7f3a/end/2",
            "reviewer",
            "notice",
            "2026-11-08T10:02:05-05:00",
        ),
        handover(
            "place/7f3a/end/2",
            "planner",
            "notice",
            "2026-11-08T10:02:06-05:00",
        ),
        confirmation("place/7f3a/end/2", "planner", "2026-11-08T10:40:12-05:00"),
    ];
    let owed = owed(&records, "2026-11-08T12:00:00-05:00").unwrap();
    assert_eq!(keys(&owed), [("reviewer", vec!["place/7f3a/end/2"])]);
    let handed: Vec<&str> = owed.readers[0].deliveries[0]
        .handovers
        .iter()
        .map(|handover| handover.reader.as_str())
        .collect();
    assert_eq!(handed, ["reviewer"], "its hand-overs to that reader alone");
    assert!(owed.done.is_empty());
}

#[test]
fn deliveries_in_the_order_recorded() {
    let records = [
        delivery("c", &["planner"], 604800),
        delivery("a", &["planner", "reviewer"], 604800),
        delivery("b", &["planner"], 604800),
        delivery("a", &["stranger"], 60),
        handover("place/0000/end/1", "planner", "notice", AT),
        confirmation("place/0000/end/1", "planner", AT),
    ];
    let owed = owed(&records, "2026-11-08T12:00:00-05:00").unwrap();
    assert_eq!(
        keys(&owed),
        [("planner", vec!["c", "a", "b"]), ("reviewer", vec!["a"])],
        "the first delivery under a key alone, and what names none passed over"
    );
    assert!(owed.readers[0].deliveries[0].handovers.is_empty());
}

#[test]
fn a_call_made_twice() {
    let records = vec![
        delivery("place/7f3a/end/2", &["planner", "reviewer"], 86400),
        handover(
            "place/7f3a/end/2",
            "planner",
            "notice",
            "2026-11-08T10:02:05-05:00",
        ),
        confirmation("place/7f3a/end/2", "reviewer", "2026-11-08T10:40:12-05:00"),
        known("reviewer", "2026-11-08T10:40:12-05:00"),
    ];
    let kept = records.clone();
    let first = owed(&records, "2026-11-12T09:00:00-05:00").unwrap();
    let second = owed(&records, "2026-11-12T09:00:00-05:00").unwrap();
    assert_eq!(first, second);
    assert_eq!(records, kept, "the records passed are unchanged");
    let refused = owed(&records, "now").unwrap_err();
    assert_eq!(refused.field, "now");
}
