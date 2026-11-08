//! A delivery's records in recutils, held to delivery-records' "The records
//! are spelled in recutils" and "A record out of its format is refused"
//! (40; C.8; core.22, core.24, core.52).

use std::io::Write as _;
use std::process::{Command, Stdio};

use weir_delivery::{
    rec, Confirmation, Delivery, Handover, Key, Known, Message, Reader, Record, Span, Time, Way,
};

/// What `recfix --check` says of `text` where it refuses it, run as
/// weir-branch's tests run it (core.24).
fn recfix_says(text: &str) -> Option<String> {
    let mut child = Command::new("recfix")
        .arg("--check")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("recfix is not on PATH: {error}"));
    child
        .stdin
        .take()
        .expect("recfix's standard input")
        .write_all(text.as_bytes())
        .expect("recfix's standard input written");
    let output = child.wait_with_output().expect("recfix ran");
    (!output.status.success()).then(|| String::from_utf8_lossy(&output.stderr).into_owned())
}

fn at(text: &str) -> Time {
    Time::parse(text, "at").unwrap()
}

fn four() -> Vec<Record> {
    let key = Key::parse("place/7f3a/end/2").unwrap();
    let planner = Reader::parse("planner").unwrap();
    vec![
        Delivery::new(
            key.clone(),
            Message::parse("construct ended: the change is written.\n\nRead its tasks next.")
                .unwrap(),
            vec![planner.clone(), Reader::parse("reviewer").unwrap()],
            at("2026-11-08T10:02:00-05:00"),
            Span::seconds(604800).unwrap(),
        )
        .unwrap()
        .into(),
        Handover {
            key: key.clone(),
            reader: planner.clone(),
            by: Way::parse("notice").unwrap(),
            at: at("2026-11-08T10:02:05-05:00"),
        }
        .into(),
        Confirmation {
            key,
            reader: planner.clone(),
            at: at("2026-11-08T10:40:12-05:00"),
        }
        .into(),
        Known {
            reader: planner,
            known_at: at("2026-11-08T10:40:12-05:00"),
        }
        .into(),
    ]
}

#[test]
fn four_records_spelled_and_read_back() {
    let records = four();
    let text = rec::write(&records);
    assert_eq!(
        text,
        "format: weir-delivery/1\n\
         key: place/7f3a/end/2\n\
         reader: planner\n\
         reader: reviewer\n\
         message: construct ended: the change is written.\n\
         +\n\
         + Read its tasks next.\n\
         recorded_at: 2026-11-08T10:02:00-05:00\n\
         expires_at: 2026-11-15T10:02:00-05:00\n\
         \n\
         format: weir-handover/1\n\
         key: place/7f3a/end/2\n\
         reader: planner\n\
         by: notice\n\
         at: 2026-11-08T10:02:05-05:00\n\
         \n\
         format: weir-confirmation/1\n\
         key: place/7f3a/end/2\n\
         reader: planner\n\
         at: 2026-11-08T10:40:12-05:00\n\
         \n\
         format: weir-known/1\n\
         reader: planner\n\
         known_at: 2026-11-08T10:40:12-05:00\n"
    );
    assert_eq!(rec::read(&text).unwrap(), records);
    if let Some(said) = recfix_says(&text) {
        panic!("recfix refused:\n{text}\n{said}");
    }
    assert_eq!(rec::read("").unwrap(), []);
}

#[test]
fn a_record_of_a_format_the_crate_does_not_read() {
    let text = "format: weir-delivery/2\nkey: place/7f3a/end/2\n";
    let refused = rec::read(text).unwrap_err();
    assert_eq!(refused.field, "format");
    assert!(refused.reason.contains("weir-delivery/2"), "{refused}");
    let none = rec::read("key: place/7f3a/end/2\n").unwrap_err();
    assert_eq!(none.field, "format");
}

#[test]
fn a_field_the_format_does_not_name() {
    let text = "format: weir-confirmation/1\nkey: place/7f3a/end/2\nreader: planner\n\
                at: 2026-11-08T10:40:12-05:00\nnote: read twice\n";
    let refused = rec::read(text).unwrap_err();
    assert_eq!(refused.field, "note");
}

#[test]
fn a_field_missing_or_given_twice() {
    let missing = "format: weir-handover/1\nkey: place/7f3a/end/2\nreader: planner\n\
                   at: 2026-11-08T10:02:05-05:00\n";
    assert_eq!(rec::read(missing).unwrap_err().field, "by");
    let twice = "format: weir-known/1\nreader: planner\nreader: reviewer\n\
                 known_at: 2026-11-08T10:40:12-05:00\n";
    let refused = rec::read(twice).unwrap_err();
    assert_eq!(refused.field, "reader");
    assert!(refused.reason.contains("more than once"), "{refused}");
    let no_reader = "format: weir-delivery/1\nkey: k\nmessage: m\n\
                     recorded_at: 2026-11-08T10:02:00-05:00\nexpires_at: 2026-11-08T10:03:00-05:00\n";
    assert_eq!(rec::read(no_reader).unwrap_err().field, "reader");
    let bad_time =
        "format: weir-confirmation/1\nkey: k\nreader: planner\nat: 2026-11-08T10:40:12\n";
    assert_eq!(rec::read(bad_time).unwrap_err().field, "at");
}
