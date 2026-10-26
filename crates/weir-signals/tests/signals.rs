//! Signals and readings, held to the scenarios of inbox-signals: the
//! record `weir-signal/1`, the own signal written with its capture, and a
//! reader's one reading (19, 25 to 28).

mod common;

use weir_capture::rec::{self, Record};
use weir_capture::{LeftOut, Nature};
use weir_signals::signal::{signals_rec, EMPTY_READING};
use weir_signals::{InboxAt, Signal, SignalId};

use common::{capture, recfix, recsel_value};

fn id(text: &str) -> SignalId {
    SignalId::parse(text).unwrap()
}

fn a_signal(signal: &str) -> Signal {
    Signal {
        signal: id(signal),
        kind: Nature::Ask,
        asserted_by: "jdx/mise".into(),
        assertion: "mise 2026.10.10 is released; the repository pins 2026.9.18.".into(),
        subjects: vec!["release".into(), "mise".into()],
        argues_with: vec!["packages/mise/default.nix".into()],
        excerpt: "## Fixes\n\n- the shim no longer times out\n".into(),
        position: "release notes".into(),
        cut: None,
    }
}

#[test]
fn a_signal_holds_every_field_in_order() {
    let signal = Signal {
        cut: Some(LeftOut {
            lines: 812,
            bytes: 39020,
        }),
        ..a_signal("release/mise/2026.10.10#1")
    };
    let written = signals_rec(std::slice::from_ref(&signal));
    assert_eq!(
        written,
        "format: weir-signal/1\n\
         signal: release/mise/2026.10.10#1\n\
         kind: ask\n\
         asserted_by: jdx/mise\n\
         assertion: mise 2026.10.10 is released; the repository pins 2026.9.18.\n\
         subjects: release\n\
         subjects: mise\n\
         argues_with: packages/mise/default.nix\n\
         excerpt: ## Fixes\n\
         +\n\
         + - the shim no longer times out\n\
         +\n\
         position: release notes\n\
         cut: 812 lines and 39020 bytes\n"
    );
    recfix(&written);
    assert_eq!(recsel_value(&written, "cut"), "812 lines and 39020 bytes");
    assert_eq!(recsel_value(&written, "excerpt"), signal.excerpt);
    let records = rec::read(&written).unwrap();
    assert_eq!(Signal::from_record(&records[0]), Ok(signal));
}

#[test]
fn empty_lists() {
    let signal = Signal {
        subjects: Vec::new(),
        argues_with: Vec::new(),
        ..a_signal("meeting/file/ab12#1")
    };
    let written = signals_rec(std::slice::from_ref(&signal));
    assert!(!written.contains("subjects:"), "{written}");
    assert!(!written.contains("argues_with:"), "{written}");
    assert!(!written.contains("cut:"), "{written}");
    recfix(&written);
    let read = Signal::from_record(&rec::read(&written).unwrap()[0]).unwrap();
    assert!(read.subjects.is_empty());
    assert!(read.argues_with.is_empty());
    assert_eq!(read, signal);
}

#[test]
fn a_signal_lacking_a_field_is_refused_naming_it() {
    let record = a_signal("release/x#1").to_record();
    for field in [
        "format",
        "signal",
        "kind",
        "asserted_by",
        "assertion",
        "excerpt",
        "position",
    ] {
        let mut lacking = Record::new();
        for (name, value) in record.fields() {
            if name != field {
                lacking.push(name, value.as_str());
            }
        }
        let error = Signal::from_record(&lacking).unwrap_err();
        assert_eq!(error.field.as_deref(), Some(field), "{error}");
    }
    let mut twice = record.clone();
    twice.push("position", "again");
    assert_eq!(
        Signal::from_record(&twice).unwrap_err().to_string(),
        "`position` is given more than once"
    );
}

#[test]
fn a_signal_of_an_unknown_nature() {
    let written = signals_rec(&[a_signal("release/a#1")]).replace("kind: ask", "kind: complaint");
    let error = Signal::from_record(&rec::read(&written).unwrap()[0]).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("kind"));
    assert_eq!(
        error.to_string(),
        "`kind` is `complaint`, not one of constraint, ask, question, commitment, reaction"
    );
    let capture = capture("release/a").to_rec();
    let error = InboxAt::read([
        ("inbox/captures/release/a/capture.rec", capture.as_str()),
        ("inbox/captures/release/a/signals.rec", written.as_str()),
    ])
    .unwrap_err();
    assert_eq!(error.path, "inbox/captures/release/a/signals.rec");
    assert_eq!(error.record, Some(1));
    assert_eq!(error.field.as_deref(), Some("kind"));
}

#[test]
fn every_nature_reads_back() {
    for kind in Nature::ALL {
        let signal = Signal {
            kind,
            ..a_signal("release/x#1")
        };
        let written = signals_rec(std::slice::from_ref(&signal));
        recfix(&written);
        assert_eq!(recsel_value(&written, "kind"), kind.as_str());
        assert_eq!(
            Signal::from_record(&rec::read(&written).unwrap()[0]),
            Ok(signal)
        );
    }
}

#[test]
fn a_cut_out_of_shape_is_refused_naming_it() {
    let written = signals_rec(&[a_signal("release/x#1")]) + "cut: most of it\n";
    let error = Signal::from_record(&rec::read(&written).unwrap()[0]).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("cut"));
}

#[test]
fn a_reading_of_none_is_one_line_naming_the_format() {
    let written = signals_rec(&[]);
    assert_eq!(written, format!("{EMPTY_READING}\n"));
    assert_eq!(written, "# weir-signal/1: read, no signals\n");
    recfix(&written);
}
