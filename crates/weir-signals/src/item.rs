//! Items, the units of work curation makes of signals, with their briefs
//! and sends (34 to 36; C.2): `weir-item/1` in `item.rec`, the brief in
//! `brief.md`, and `weir-send/1` in `sends.rec`, each send with what became
//! of it added after it. An item holds no list of its signals and no
//! weight: both are read from the standing moves (34; core.20).

use std::fmt;

use weir_capture::rec::{RecError, Record};

use crate::format::Format;
use crate::layout::check_name;
use crate::moves::{check_time, required_text};

/// What made an item (34).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MadeBy {
    /// A join naming it, by the move's maker at the move's time.
    Join,
    /// The work an accepted conclusion proposed, naming that conclusion.
    Conclusion(String),
}

/// One item, as its record holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub item: String,
    pub made_by: MadeBy,
    pub named_by: String,
    /// RFC 3339 with its offset, as the caller handed it.
    pub named_at: String,
}

impl Item {
    /// The record, `format` first, then the fields in C.2's order, with
    /// `conclusion` only where one made it.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        record.push("format", Format::Item.as_str());
        record.push("item", self.item.as_str());
        match &self.made_by {
            MadeBy::Join => record.push("made_by", "join"),
            MadeBy::Conclusion(conclusion) => {
                record.push("made_by", "conclusion");
                record.push("conclusion", conclusion.as_str());
            }
        }
        record.push("named_by", self.named_by.as_str());
        record.push("named_at", self.named_at.as_str());
        record
    }

    /// The item of a `weir-item/1` record, refused naming the field that is
    /// missing, given twice or not of its shape.
    pub fn from_record(record: &Record) -> Result<Item, RecError> {
        Format::of_record(record, &[Format::Item])?;
        let item = record.required("item")?;
        check_name(item).map_err(|reason| RecError::field("item", format!("`{item}` {reason}")))?;
        let conclusion = record.single("conclusion")?;
        let made_by = match (record.required("made_by")?, conclusion) {
            ("join", None) => MadeBy::Join,
            ("join", Some(_)) => {
                return Err(RecError::field(
                    "conclusion",
                    "is given, and a join made the item",
                ))
            }
            ("conclusion", Some(conclusion)) if !conclusion.is_empty() => {
                MadeBy::Conclusion(conclusion.to_string())
            }
            ("conclusion", _) => {
                return Err(RecError::field(
                    "conclusion",
                    "is missing, and an item made from a conclusion names it",
                ))
            }
            (other, _) => {
                return Err(RecError::field(
                    "made_by",
                    format!("is `{other}`, not join or conclusion"),
                ))
            }
        };
        let named_at = record.required("named_at")?;
        check_time(named_at).map_err(|reason| RecError::field("named_at", reason))?;
        Ok(Item {
            item: item.to_string(),
            made_by,
            named_by: required_text(record, "named_by")?,
            named_at: named_at.to_string(),
        })
    }
}

/// What became of a send (36; C.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing yet: every send is written so.
    Open,
    Abandoned,
    /// A landing, with what landed.
    Landing(String),
    /// A conclusion, with the conclusion.
    Conclusion(String),
}

impl Outcome {
    /// `open`, `abandoned`, `landing <what landed>` or
    /// `conclusion <the conclusion>`.
    pub fn parse(text: &str) -> Option<Outcome> {
        match text {
            "open" => Some(Outcome::Open),
            "abandoned" => Some(Outcome::Abandoned),
            _ => {
                let (word, what) = text.split_once(' ')?;
                if what.is_empty() {
                    return None;
                }
                match word {
                    "landing" => Some(Outcome::Landing(what.to_string())),
                    "conclusion" => Some(Outcome::Conclusion(what.to_string())),
                    _ => None,
                }
            }
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Open => f.write_str("open"),
            Outcome::Abandoned => f.write_str("abandoned"),
            Outcome::Landing(what) => write!(f, "landing {what}"),
            Outcome::Conclusion(conclusion) => write!(f, "conclusion {conclusion}"),
        }
    }
}

/// One record of `sends.rec`: a send, which holds the brief as it stood
/// and the outcome `open`, or what became of one, which names it by its
/// target and when it was sent, holds no brief, and gives the outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendRecord {
    pub item: String,
    /// The caller's text: a repository and kind, a planner or an inquiry.
    pub target: String,
    /// The brief as it stood when sent, on a send alone (36).
    pub brief: Option<String>,
    /// RFC 3339 with its offset, as the caller handed it.
    pub sent_at: String,
    pub outcome: Outcome,
}

impl SendRecord {
    /// Whether this is a send, rather than what became of one.
    pub fn is_send(&self) -> bool {
        self.brief.is_some()
    }

    /// The record, `format` first, then the fields in C.2's order.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        record.push("format", Format::Send.as_str());
        record.push("item", self.item.as_str());
        record.push("target", self.target.as_str());
        record.push_optional("brief", self.brief.as_deref());
        record.push("sent_at", self.sent_at.as_str());
        record.push("outcome", self.outcome.to_string());
        record
    }

    /// The record of `weir-send/1`, refused naming the field that is
    /// missing, given twice or not of its shape: a send's outcome is open,
    /// and what became of one is not.
    pub fn from_record(record: &Record) -> Result<SendRecord, RecError> {
        Format::of_record(record, &[Format::Send])?;
        let sent_at = record.required("sent_at")?;
        check_time(sent_at).map_err(|reason| RecError::field("sent_at", reason))?;
        let outcome_text = record.required("outcome")?;
        let outcome = Outcome::parse(outcome_text).ok_or_else(|| {
            RecError::field(
                "outcome",
                format!(
                    "is `{outcome_text}`, not open, abandoned, landing <what landed> or \
                     conclusion <the conclusion>"
                ),
            )
        })?;
        let brief = record.single("brief")?;
        match (brief, &outcome) {
            (Some(_), Outcome::Open) => {}
            (None, outcome) if *outcome != Outcome::Open => {}
            (Some(_), _) => {
                return Err(RecError::field(
                    "outcome",
                    format!("is `{outcome}`, and a send is written open; what became of it is added after it"),
                ))
            }
            (None, _) => {
                return Err(RecError::field(
                    "brief",
                    "is missing, and a send keeps the brief as it was handed",
                ))
            }
        }
        Ok(SendRecord {
            item: record.required("item")?.to_string(),
            target: required_text(record, "target")?,
            brief: brief.map(str::to_string),
            sent_at: sent_at.to_string(),
            outcome,
        })
    }
}
