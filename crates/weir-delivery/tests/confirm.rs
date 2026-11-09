//! `confirm`, held to delivery-calls' "confirm returns the confirmation to
//! add", "confirm takes a reader's confirmation once" and "confirm refuses
//! a key or a reader the records do not hold" (102, 104, 106, 110; S24;
//! core.48, core.50).

use weir_delivery::{confirm, owed, record, Confirmed, Record, Recorded};

const KEY: &str = "place/7f3a/end/2";
const AT: &str = "2026-11-08T10:02:00-05:00";
const CONFIRMED_AT: &str = "2026-11-08T10:40:12-05:00";

/// A delivery for `readers` under `KEY`, recorded at `AT` and expiring a
/// day on.
fn recorded(readers: &[&str]) -> Vec<Record> {
    match record(&[], KEY, "construct ended.", readers, AT, 86400).unwrap() {
        Recorded::Add(records) => records,
        held => panic!("{held:?}"),
    }
}

fn added(confirmed: Confirmed) -> Vec<Record> {
    match confirmed {
        Confirmed::Add(records) => records,
        already => panic!("{already:?}"),
    }
}

#[test]
fn a_reader_confirms_by_its_key() {
    let records = recorded(&["planner", "reviewer"]);
    let add = added(confirm(&records, KEY, "planner", CONFIRMED_AT).unwrap());
    let [Record::Confirmation(confirmation), Record::Known(known)] = add.as_slice() else {
        panic!("{add:?}");
    };
    assert_eq!(
        (
            confirmation.key.as_str(),
            confirmation.reader.as_str(),
            confirmation.at.as_str()
        ),
        (KEY, "planner", CONFIRMED_AT)
    );
    assert_eq!(
        (known.reader.as_str(), known.known_at.as_str()),
        ("planner", CONFIRMED_AT)
    );
    let mut with = records.clone();
    with.extend(add);
    let add = added(confirm(&with, KEY, "reviewer", CONFIRMED_AT).unwrap());
    assert_eq!(add.len(), 2, "`reviewer` is not yet known");
    let mut known_before = records;
    known_before.push(with[2].clone());
    let add = added(confirm(&known_before, KEY, "planner", CONFIRMED_AT).unwrap());
    assert!(
        matches!(add.as_slice(), [Record::Confirmation(_)]),
        "a name known adds no weir-known/1: {add:?}"
    );
}

#[test]
fn a_second_confirmation() {
    let mut records = recorded(&["planner"]);
    records.extend(added(
        confirm(&records, KEY, "planner", CONFIRMED_AT).unwrap(),
    ));
    let again = confirm(&records, KEY, "planner", "2026-11-09T08:00:00-05:00").unwrap();
    assert_eq!(
        again,
        Confirmed::Already(weir_delivery::Time::parse(CONFIRMED_AT, "at").unwrap())
    );
}

#[test]
fn a_confirmation_of_a_key_no_delivery_is_under() {
    let records = recorded(&["planner"]);
    let refused = confirm(&records, "place/0000/end/1", "planner", CONFIRMED_AT).unwrap_err();
    assert_eq!(refused.field, "key");
    assert!(refused.reason.contains("place/0000/end/1"), "{refused}");
}

#[test]
fn a_confirmation_by_a_reader_not_named() {
    let records = recorded(&["planner"]);
    let refused = confirm(&records, KEY, "stranger", CONFIRMED_AT).unwrap_err();
    assert_eq!(refused.field, "reader");
    assert!(refused.reason.contains("stranger"), "{refused}");
    let out_of_shape = confirm(&records, KEY, "planner", "2026-11-08T10:40:12").unwrap_err();
    assert_eq!(out_of_shape.field, "at");
}

#[test]
fn an_expired_delivery_confirmed() {
    let mut records = recorded(&["planner"]);
    let late = "2026-11-12T09:00:00-05:00";
    assert!(owed(&records, late).unwrap().readers[0].deliveries[0].expired);
    records.extend(added(confirm(&records, KEY, "planner", late).unwrap()));
    let owed = owed(&records, late).unwrap();
    assert!(owed.readers.is_empty(), "{owed:?}");
    assert_eq!(owed.done.len(), 1);
}
