//! A signal, one assertion read from a capture, as `weir-signal/1` records
//! it in its capture's `signals.rec` (25, 27; C.2): the capture's own signal,
//! or the reading a reader writes for it, numbered from 1.

use std::fmt;

use weir_capture::rec::{self, RecError, Record};
use weir_capture::{Key, LeftOut, Nature, OwnSignal};

use crate::format::Format;
use crate::moves::required_text;

/// What a `signals.rec` holds for a reading of no signals: no record, and
/// this one line, which names the format and which `recfix` takes. Its
/// presence is what makes the capture read (28).
pub const EMPTY_READING: &str = "# weir-signal/1: read, no signals";

/// A signal's identity, `<key>#<n>`: its capture's key and its number in
/// reading order, from 1 (27).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId {
    pub key: Key,
    pub n: u64,
}

impl SignalId {
    pub fn new(key: Key, n: u64) -> SignalId {
        SignalId { key, n }
    }

    /// The identity `text` names: a key the key rule takes, `#`, and a
    /// number from 1, in decimal with no leading zero.
    pub fn parse(text: &str) -> Result<SignalId, String> {
        let (key, n) = text
            .rsplit_once('#')
            .ok_or_else(|| format!("`{text}` is not a signal's identity, `<key>#<n>`"))?;
        let key =
            Key::parse(key).map_err(|error| format!("`{text}` names no capture's key: {error}"))?;
        let n = count(n).ok_or_else(|| {
            format!("`{text}` numbers no signal: a signal's number is a count from 1")
        })?;
        Ok(SignalId { key, n })
    }
}

impl fmt::Display for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.key, self.n)
    }
}

/// `text` as a count from 1, written in decimal with no sign and no
/// leading zero.
pub(crate) fn count(text: &str) -> Option<u64> {
    text.parse::<u64>()
        .ok()
        .filter(|n| *n >= 1 && n.to_string() == text)
}

/// One signal, as its record holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    pub signal: SignalId,
    /// Its nature, in the field `kind` (25; record 0002).
    pub kind: Nature,
    pub asserted_by: String,
    pub assertion: String,
    pub subjects: Vec<String>,
    pub argues_with: Vec<String>,
    /// As it stands in the capture's raw material, or the own excerpt as
    /// cut (26).
    pub excerpt: String,
    /// Where the excerpt stands.
    pub position: String,
    /// What the cut left out, on a cut own signal alone (20).
    pub cut: Option<LeftOut>,
}

impl Signal {
    /// The capture crate's own signal, numbered `<key>#1` (19, 27).
    pub fn own(own: &OwnSignal) -> Result<Signal, String> {
        Ok(Signal {
            signal: SignalId::parse(&own.signal)?,
            kind: own.kind,
            asserted_by: own.asserted_by.clone(),
            assertion: own.assertion.clone(),
            subjects: own.subjects.clone(),
            argues_with: own.argues_with.clone(),
            excerpt: own.excerpt.clone(),
            position: own.position.clone(),
            cut: own.cut,
        })
    }

    /// The record, `format` first, then the fields in C.2's order: a list
    /// as one field per entry and none for an empty one, `cut` only where
    /// the excerpt was cut.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        record.push("format", Format::Signal.as_str());
        record.push("signal", self.signal.to_string());
        record.push("kind", self.kind.as_str());
        record.push("asserted_by", self.asserted_by.as_str());
        record.push("assertion", self.assertion.as_str());
        record.push_list("subjects", &self.subjects);
        record.push_list("argues_with", &self.argues_with);
        record.push("excerpt", self.excerpt.as_str());
        record.push("position", self.position.as_str());
        if let Some(left_out) = self.cut {
            record.push("cut", spell_cut(left_out));
        }
        record
    }

    /// The signal of a `weir-signal/1` record, refused naming the field
    /// that is missing, given twice or not of its shape. A text field left
    /// empty is lacking, as a reading's is (25); the lists may be empty.
    pub fn from_record(record: &Record) -> Result<Signal, RecError> {
        Format::of_record(record, &[Format::Signal])?;
        let signal = record.required("signal")?;
        let signal = SignalId::parse(signal).map_err(|reason| RecError::field("signal", reason))?;
        let kind = record.required("kind")?;
        let kind =
            Nature::parse(kind).ok_or_else(|| RecError::field("kind", not_a_nature(kind)))?;
        let cut = match record.single("cut")? {
            None => None,
            Some(text) => Some(read_cut(text).ok_or_else(|| {
                RecError::field("cut", format!("is `{text}`, not `<n> lines and <b> bytes`"))
            })?),
        };
        Ok(Signal {
            signal,
            kind,
            asserted_by: required_text(record, "asserted_by")?,
            assertion: required_text(record, "assertion")?,
            subjects: strings(record.get_all("subjects")),
            argues_with: strings(record.get_all("argues_with")),
            excerpt: required_text(record, "excerpt")?,
            position: required_text(record, "position")?,
            cut,
        })
    }
}

fn strings(values: Vec<&str>) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

pub(crate) fn not_a_nature(kind: &str) -> String {
    format!(
        "is `{kind}`, not one of {}",
        Nature::ALL.map(Nature::as_str).join(", ")
    )
}

/// What a cut left out, as C.1's closing line spells it:
/// `<n> lines and <b> bytes`.
pub fn spell_cut(left_out: LeftOut) -> String {
    format!("{} lines and {} bytes", left_out.lines, left_out.bytes)
}

fn read_cut(text: &str) -> Option<LeftOut> {
    let (lines, bytes) = text.strip_suffix(" bytes")?.split_once(" lines and ")?;
    let number = |text: &str| text.parse::<u64>().ok().filter(|n| n.to_string() == text);
    Some(LeftOut {
        lines: number(lines)?,
        bytes: number(bytes)?,
    })
}

/// The text of a `signals.rec` holding `signals`, or, for a reading of
/// none, [`EMPTY_READING`] alone.
pub fn signals_rec(signals: &[Signal]) -> String {
    if signals.is_empty() {
        return format!("{EMPTY_READING}\n");
    }
    let records: Vec<Record> = signals.iter().map(Signal::to_record).collect();
    rec::write(&records)
}

/// A signal as a reader hands it, before it is numbered and checked: every
/// field as text, so that a nature not of the five can be named.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadSignal {
    pub kind: String,
    pub asserted_by: String,
    pub assertion: String,
    pub subjects: Vec<String>,
    pub argues_with: Vec<String>,
    pub excerpt: String,
    pub position: String,
}

impl ReadSignal {
    /// The signal `id` this reads as, or each field it lacks or holds out
    /// of shape, with why. A text field left empty is lacking (25); the
    /// lists may be empty.
    pub fn to_signal(&self, id: SignalId) -> Result<Signal, Vec<(String, String)>> {
        let mut failures = Vec::new();
        for (field, value) in [
            ("kind", &self.kind),
            ("asserted_by", &self.asserted_by),
            ("assertion", &self.assertion),
            ("excerpt", &self.excerpt),
            ("position", &self.position),
        ] {
            if value.is_empty() {
                failures.push((field.to_string(), "is missing".to_string()));
            }
        }
        let kind = Nature::parse(&self.kind);
        if kind.is_none() && !self.kind.is_empty() {
            failures.push(("kind".to_string(), not_a_nature(&self.kind)));
        }
        match kind {
            Some(kind) if failures.is_empty() => Ok(Signal {
                signal: id,
                kind,
                asserted_by: self.asserted_by.clone(),
                assertion: self.assertion.clone(),
                subjects: self.subjects.clone(),
                argues_with: self.argues_with.clone(),
                excerpt: self.excerpt.clone(),
                position: self.position.clone(),
                cut: None,
            }),
            _ => Err(failures),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identity_reads_back_as_written() {
        let id = SignalId::parse("release/mise/2026.10.10#12").unwrap();
        assert_eq!(id.key.as_str(), "release/mise/2026.10.10");
        assert_eq!(id.n, 12);
        assert_eq!(id.to_string(), "release/mise/2026.10.10#12");
        // A key may hold `#`: the number is after the last.
        let id = SignalId::parse("folder/a#b#3").unwrap();
        assert_eq!((id.key.as_str(), id.n), ("folder/a#b", 3));
        for text in [
            "release/x",
            "release/x#",
            "release/x#0",
            "release/x#01",
            "release/x#-1",
            "#1",
            "moves/x#1",
        ] {
            assert!(SignalId::parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn a_cut_is_spelled_as_the_closing_line_spells_it() {
        let left_out = LeftOut {
            lines: 812,
            bytes: 39020,
        };
        assert_eq!(spell_cut(left_out), "812 lines and 39020 bytes");
        assert_eq!(read_cut("812 lines and 39020 bytes"), Some(left_out));
        for text in [
            "812 lines",
            "812 lines and 39 020 bytes",
            "0812 lines and 1 bytes",
            "1 line and 1 byte",
        ] {
            assert_eq!(read_cut(text), None, "{text}");
        }
    }

    #[test]
    fn a_read_signal_names_each_field_it_lacks() {
        let failures = ReadSignal {
            kind: "complaint".into(),
            excerpt: "x".into(),
            position: "line 1".into(),
            ..ReadSignal::default()
        }
        .to_signal(SignalId::parse("release/x#1").unwrap())
        .unwrap_err();
        let fields: Vec<&str> = failures.iter().map(|(field, _)| field.as_str()).collect();
        assert_eq!(fields, ["asserted_by", "assertion", "kind"]);
        assert!(failures[2].1.contains("`complaint`, not one of"));
    }
}
