//! `hand`, held to delivery-calls' "hand returns a hand-over, which
//! confirms nothing by itself", "A reader confirms on receipt in the call
//! that hands it over" and "hand hands nothing to a reader that confirmed
//! or is not named" (102, 103, 109; S22; core.48).

use weir_delivery::{confirm, hand, owed, record, Confirmed, Handed, Record, Recorded, Time};

const KEY: &str = "place/7f3a/end/2";
const AT: &str = "2026-11-08T10:02:00-05:00";
const HANDED_AT: &str = "2026-11-08T10:02:05-05:00";

fn recorded(readers: &[&str]) -> Vec<Record> {
    match record(&[], KEY, "construct ended.", readers, AT, 604800).unwrap() {
        Recorded::Add(records) => records,
        held => panic!("{held:?}"),
    }
}

fn added(handed: Handed) -> Vec<Record> {
    match handed {
        Handed::Add(records) => records,
        confirmed => panic!("{confirmed:?}"),
    }
}

#[test]
fn a_hand_over_by_a_notice() {
    let mut records = recorded(&["planner"]);
    let add = added(hand(&records, KEY, "planner", "notice", HANDED_AT, false).unwrap());
    let [Record::Handover(handover)] = add.as_slice() else {
        panic!("{add:?}");
    };
    assert_eq!(
        (
            handover.key.as_str(),
            handover.reader.as_str(),
            handover.by.as_str(),
            handover.at.as_str()
        ),
        (KEY, "planner", "notice", HANDED_AT)
    );
    records.extend(add.clone());
    let owed = owed(&records, "2026-11-08T11:00:00-05:00").unwrap();
    let [planner] = owed.readers.as_slice() else {
        panic!("{owed:?}");
    };
    assert_eq!(planner.reader.as_str(), "planner");
    assert_eq!(
        planner.deliveries[0].handovers,
        [handover.clone()],
        "still owed, with that hand-over"
    );
    assert!(owed.done.is_empty());
}

#[test]
fn a_reader_that_confirms_on_receipt() {
    let records = recorded(&["planner", "reviewer"]);
    let add = added(hand(&records, KEY, "reviewer", "session-start", HANDED_AT, true).unwrap());
    let formats: Vec<&str> = add.iter().map(Record::format).collect();
    assert_eq!(
        formats,
        ["weir-handover/1", "weir-confirmation/1", "weir-known/1"]
    );
    let Confirmed::Add(confirmed) = confirm(&records, KEY, "reviewer", HANDED_AT).unwrap() else {
        panic!("a confirmation to add");
    };
    assert_eq!(
        add[1..],
        confirmed[..],
        "what confirm gives, after the hand-over"
    );
}

#[test]
fn a_hand_over_to_a_reader_that_has_confirmed() {
    let mut records = recorded(&["planner"]);
    let confirmed_at = "2026-11-08T10:40:12-05:00";
    let Confirmed::Add(add) = confirm(&records, KEY, "planner", confirmed_at).unwrap() else {
        panic!("a confirmation to add");
    };
    records.extend(add);
    for confirms in [false, true] {
        let handed = hand(
            &records,
            KEY,
            "planner",
            "wait",
            "2026-11-09T08:00:00-05:00",
            confirms,
        )
        .unwrap();
        assert_eq!(
            handed,
            Handed::Confirmed(Time::parse(confirmed_at, "at").unwrap())
        );
    }
}

#[test]
fn a_hand_over_to_a_reader_the_delivery_does_not_name() {
    let records = recorded(&["planner"]);
    let refused = hand(&records, KEY, "stranger", "notice", HANDED_AT, false).unwrap_err();
    assert_eq!(refused.field, "reader");
    assert!(refused.reason.contains("stranger"), "{refused}");
    let no_key = hand(
        &records,
        "place/0000/end/1",
        "planner",
        "notice",
        HANDED_AT,
        false,
    )
    .unwrap_err();
    assert_eq!(no_key.field, "key");
    let no_way = hand(&records, KEY, "planner", " notice", HANDED_AT, false).unwrap_err();
    assert_eq!(no_way.field, "by");
}
