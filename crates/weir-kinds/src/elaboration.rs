use std::fmt;

use toml::Value;

use crate::fields::{boolean, field, key, strings, unnamed};
use crate::kinds::Kind;
use crate::refusal::{Problems, Refusal};

/// The fields of an elaboration section the format names; every other key
/// is a section of its own.
const ELABORATION_FIELDS: [&str; 5] = ["inputs", "conclusion", "ending", "transcripts", "lands"];

/// An elaboration kind's section, read by whoever runs elaborations (55,
/// C.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Elaboration {
    /// The inputs a round is handed beyond its opener's; none when left
    /// out (55).
    pub inputs: Vec<String>,
    /// The sections a conclusion adds; none when left out (55).
    pub conclusion: Vec<String>,
    /// How a round ends; `None` when left out, since the format gives no
    /// default and what it means left out is its reader's (55, 58).
    pub ending: Option<Ending>,
    /// Whether earlier rounds' transcripts are handed; false when left out
    /// (55).
    pub transcripts: bool,
    /// What lands; nothing when left out (55).
    pub lands: Vec<String>,
    /// Every key of the section the format does not name, unread and as
    /// written (56).
    pub sections: toml::Table,
}

/// How a round of an elaboration kind ends (55, C.4). Its
/// [`Display`](fmt::Display) is its spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// `self-closing`.
    SelfClosing,
    /// `standing`.
    Standing,
    /// `with-operator`.
    WithOperator,
}

impl Ending {
    fn from_spelling(spelling: &str) -> Option<Self> {
        match spelling {
            "self-closing" => Some(Self::SelfClosing),
            "standing" => Some(Self::Standing),
            "with-operator" => Some(Self::WithOperator),
            _ => None,
        }
    }
}

impl fmt::Display for Ending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SelfClosing => "self-closing",
            Self::Standing => "standing",
            Self::WithOperator => "with-operator",
        })
    }
}

/// The elaboration read of a kind (55): its `elaboration` section, or
/// `None` when it has none, checked whole and refused naming every field
/// that fails it (57).
///
/// [`read_kinds`](crate::read_kinds) returns the section unread among the
/// kind's sections and never fails on it, so whoever runs places reads the
/// kind's stages whatever the section holds (56).
pub fn read_elaboration(kind: &Kind) -> Result<Option<Elaboration>, Refusal> {
    let Some(section) = kind.sections.get("elaboration") else {
        return Ok(None);
    };
    let at = field(&format!("kinds.{}", key(&kind.name)), "elaboration");
    let mut problems = Problems::default();
    let Value::Table(section) = section else {
        problems.refuse(&at, "must be a table");
        return problems.finish(None);
    };
    let inputs = strings(section, "inputs", &at, &mut problems);
    let conclusion = strings(section, "conclusion", &at, &mut problems);
    let ending = match section.get("ending") {
        None => None,
        Some(Value::String(spelling)) => {
            let ending = Ending::from_spelling(spelling);
            if ending.is_none() {
                problems.refuse(
                    &field(&at, "ending"),
                    format!("`{spelling}` is not one of self-closing, standing and with-operator"),
                );
            }
            ending
        }
        Some(_) => {
            problems.refuse(
                &field(&at, "ending"),
                "must be one of self-closing, standing and with-operator",
            );
            None
        }
    };
    let transcripts = boolean(section, "transcripts", &at, &mut problems);
    let lands = strings(section, "lands", &at, &mut problems);
    problems.finish(Some(Elaboration {
        inputs,
        conclusion,
        ending,
        transcripts,
        lands,
        sections: unnamed(section, &ELABORATION_FIELDS),
    }))
}
