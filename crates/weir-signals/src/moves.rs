//! Moves, curation's standing judgment on a signal, as `weir-move/1`
//! records them in `moves.rec` (29 to 31; C.2): one of six words, each with
//! its consequence, who made it and when, and the batch it stood in. A
//! signal's standing move is its last; a replacement names, as `replaces`,
//! the number of the move it replaces among its signal's moves, from 1.

use std::fmt;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::rec::{RecError, Record};

use crate::format::Format;
use crate::layout::check_name;
use crate::signal::{count, SignalId};

/// The six words a move is one of (30).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Word {
    Drop,
    Answered,
    Challenge,
    Join,
    Attach,
    Route,
}

impl Word {
    pub const ALL: [Word; 6] = [
        Word::Drop,
        Word::Answered,
        Word::Challenge,
        Word::Join,
        Word::Attach,
        Word::Route,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Word::Drop => "drop",
            Word::Answered => "answered",
            Word::Challenge => "challenge",
            Word::Join => "join",
            Word::Attach => "attach",
            Word::Route => "route",
        }
    }

    pub fn parse(text: &str) -> Option<Word> {
        Word::ALL.into_iter().find(|word| word.as_str() == text)
    }

    /// The consequence the word carries in `target`, as 30 names it.
    pub fn consequence(self) -> &'static str {
        match self {
            Word::Drop => "its reason",
            Word::Answered => "the claim or record that settled the signal",
            Word::Challenge => "the claim it weighs against",
            Word::Join => "the item it makes or grows",
            Word::Attach => "the sent item it adds to",
            Word::Route => "the chore or the ask it offers",
        }
    }

    /// Whether its consequence is an item, by name (34).
    pub fn names_an_item(self) -> bool {
        matches!(self, Word::Join | Word::Attach)
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One move, as its record holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub signal: SignalId,
    pub word: Word,
    /// The consequence: a reason, a claim or record, an item, a chore or an
    /// ask (30).
    pub target: String,
    pub by: String,
    /// RFC 3339 with its offset, as the caller handed it (31).
    pub at: String,
    /// The batch it stood in, where there was one (31).
    pub batch: Option<String>,
    /// On a replacement, the number of the move it replaces among its
    /// signal's moves, counting from 1 (29).
    pub replaces: Option<u64>,
}

impl Move {
    /// The record, `format` first, then the fields in C.2's order.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        record.push("format", Format::Move.as_str());
        record.push("signal", self.signal.to_string());
        record.push("move", self.word.as_str());
        record.push("target", self.target.as_str());
        record.push("by", self.by.as_str());
        record.push("at", self.at.as_str());
        record.push_optional("batch", self.batch.as_deref());
        if let Some(replaces) = self.replaces {
            record.push("replaces", replaces.to_string());
        }
        record
    }

    /// The move of a `weir-move/1` record, refused naming the field that is
    /// missing, given twice or not of its shape: a word not of the six or a
    /// consequence missing among them (30).
    pub fn from_record(record: &Record) -> Result<Move, RecError> {
        Format::of_record(record, &[Format::Move])?;
        let signal = record.required("signal")?;
        let signal = SignalId::parse(signal).map_err(|reason| RecError::field("signal", reason))?;
        let word = shape(
            record.required("move")?,
            record.single("target")?.unwrap_or_default(),
            record.single("by")?.unwrap_or_default(),
            record.single("at")?.unwrap_or_default(),
        )
        .map_err(|(field, reason)| RecError::field(field, reason))?;
        let batch = record.single("batch")?;
        if let Some(batch) = batch {
            check_name(batch)
                .map_err(|reason| RecError::field("batch", format!("`{batch}` {reason}")))?;
        }
        let replaces = match record.single("replaces")? {
            None => None,
            Some(text) => Some(count(text).ok_or_else(|| {
                RecError::field(
                    "replaces",
                    format!("is `{text}`, not a move's number, a count from 1"),
                )
            })?),
        };
        Ok(Move {
            signal,
            word,
            target: record.required("target")?.to_string(),
            by: record.required("by")?.to_string(),
            at: record.required("at")?.to_string(),
            batch: batch.map(str::to_string),
            replaces,
        })
    }

    /// Itself, numbered among its signal's moves, as a refusal names it.
    pub(crate) fn describe(&self, number: u64) -> String {
        format!(
            "move {number} of `{}`, a {} by {} at {}",
            self.signal, self.word, self.by, self.at
        )
    }
}

/// A move as a caller hands it, before it is checked: its word as text, so
/// that a word not of the six can be named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewMove {
    pub signal: SignalId,
    pub word: String,
    pub target: String,
    pub by: String,
    pub at: String,
    /// The number of the standing move it replaces, on a replacement (29).
    pub replaces: Option<u64>,
}

impl NewMove {
    /// The move this is, in `batch` where it is proposed in one, or the
    /// field out of its shape and why.
    pub(crate) fn to_move(&self, batch: Option<&str>) -> Result<Move, (String, String)> {
        let word = shape(&self.word, &self.target, &self.by, &self.at)
            .map_err(|(field, reason)| (field.to_string(), reason))?;
        Ok(Move {
            signal: self.signal.clone(),
            word,
            target: self.target.clone(),
            by: self.by.clone(),
            at: self.at.clone(),
            batch: batch.map(str::to_string),
            replaces: self.replaces,
        })
    }
}

/// The word of a move, refused naming the field when it is not one of the
/// six, its consequence is missing or not an item's name where it is one,
/// or it names no maker or no time (30, 31).
fn shape(word: &str, target: &str, by: &str, at: &str) -> Result<Word, (&'static str, String)> {
    let parsed = Word::parse(word).ok_or_else(|| {
        (
            "move",
            format!(
                "is `{word}`, not one of {}",
                Word::ALL.map(Word::as_str).join(", ")
            ),
        )
    })?;
    if target.is_empty() {
        return Err((
            "target",
            format!(
                "is missing, and a {parsed} carries {}",
                parsed.consequence()
            ),
        ));
    }
    if parsed.names_an_item() {
        check_name(target).map_err(|reason| {
            (
                "target",
                format!("is `{target}`, which is no item's name: it {reason}"),
            )
        })?;
    }
    if by.is_empty() {
        return Err(("by", "is missing, and a move names who made it".to_string()));
    }
    check_time(at).map_err(|reason| ("at", reason))?;
    Ok(parsed)
}

/// That `text` is a time in RFC 3339 with its offset, as every time an
/// inbox's records hold is (core.14).
pub(crate) fn check_time(text: &str) -> Result<(), String> {
    OffsetDateTime::parse(text, &Rfc3339)
        .map(|_| ())
        .map_err(|_| format!("is `{text}`, not an RFC 3339 time with its offset"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_reads_back() {
        for word in Word::ALL {
            assert_eq!(Word::parse(word.as_str()), Some(word));
        }
        assert_eq!(Word::parse("promote"), None);
        assert_eq!(Word::parse("Drop"), None);
    }

    #[test]
    fn a_time_with_no_offset_is_refused() {
        assert_eq!(check_time("2026-10-25T09:00:00Z"), Ok(()));
        assert_eq!(check_time("2026-10-25T11:00:00+02:00"), Ok(()));
        for text in ["2026-10-25T09:00:00", "2026-10-25", "", "yesterday"] {
            assert!(check_time(text).is_err(), "{text}");
        }
    }
}
