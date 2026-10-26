//! The index of the statement's clause numbers the pack keeps at
//! `clauses.toml`: the commit of the statement it was read from, and each
//! section of Parts A and B with the numbers it holds, numbers only
//! (core.37). The pack's check reads it in place of the statement, which
//! lies in another repository, so the check needs no source (67, 68).

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use toml::Value;

/// Where the index lies in a pack.
pub const INDEX: &str = "clauses.toml";

/// A clause, by its number: a whole number, `14`, or a number with its
/// letter, `75a`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clause {
    number: u32,
    letter: Option<char>,
}

impl Clause {
    /// The clause numbered `number`.
    pub fn number(number: u32) -> Clause {
        Clause {
            number,
            letter: None,
        }
    }

    /// The clause `text` names: `14` or `75a`, or none.
    pub fn parse(text: &str) -> Option<Clause> {
        let digits = text.trim_end_matches(|c: char| c.is_ascii_lowercase());
        let letters = &text[digits.len()..];
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) || letters.len() > 1 {
            return None;
        }
        let number: u32 = digits.parse().ok().filter(|n| *n > 0)?;
        Some(Clause {
            number,
            letter: letters.chars().next(),
        })
    }

    /// The clause a cited TOML value names: a whole number, or a string
    /// naming a number with its letter; or why it names none.
    pub fn cited(value: &Value) -> Result<Clause, String> {
        match value {
            Value::Integer(n) => u32::try_from(*n)
                .ok()
                .filter(|n| *n > 0)
                .map(Clause::number)
                .ok_or_else(|| format!("`{n}` is no clause's number")),
            Value::String(text) => {
                Clause::parse(text).ok_or_else(|| format!("`{text:?}` is no clause's number"))
            }
            other => Err(format!("`{other}` is no clause's number")),
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number)?;
        if let Some(letter) = self.letter {
            write!(f, "{letter}")?;
        }
        Ok(())
    }
}

/// In the statement's order: by number, then a lettered clause after the
/// one it follows, `75` before `75a`.
impl Ord for Clause {
    fn cmp(&self, other: &Clause) -> Ordering {
        (self.number, self.letter).cmp(&(other.number, other.letter))
    }
}

impl PartialOrd for Clause {
    fn partial_cmp(&self, other: &Clause) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// One section of the statement and the clauses it holds, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// `A.1` to `A.12`, `B.1` to `B.6`.
    pub name: String,
    pub clauses: Vec<Clause>,
}

/// The index read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    /// The statement, as `<repository>:<path>`.
    pub statement: String,
    /// The statement's commit the numbers were read from.
    pub commit: String,
    pub sections: Vec<Section>,
}

/// The sections whose clauses the pack's check counts, with 71 beside
/// them (67).
pub const COUNTED_SECTIONS: [&str; 10] = [
    "A.1", "A.2", "A.3", "A.4", "A.5", "A.6", "A.7", "A.8", "A.9", "A.10",
];

/// The clause of A.12 the check counts beside the sections (67).
pub const COUNTED_BESIDE: u32 = 71;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    statement: String,
    commit: String,
    section: Vec<WrittenSection>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenSection {
    name: String,
    clauses: Vec<Value>,
}

impl Index {
    /// Whether the index holds `clause`.
    pub fn holds(&self, clause: &Clause) -> bool {
        self.sections
            .iter()
            .any(|section| section.clauses.contains(clause))
    }

    /// The section `name`, where the index holds it.
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.name == name)
    }

    /// The clauses the pack's check counts: every clause of A.1 to A.10,
    /// and 71 (67); or why the index cannot say, naming the section or the
    /// clause it lacks.
    pub fn counted(&self) -> Result<BTreeSet<Clause>, String> {
        let mut counted = BTreeSet::new();
        for name in COUNTED_SECTIONS {
            let section = self
                .section(name)
                .ok_or_else(|| format!("{INDEX}: holds no section {name}"))?;
            counted.extend(section.clauses.iter().cloned());
        }
        let beside = Clause::number(COUNTED_BESIDE);
        if !self.holds(&beside) {
            return Err(format!("{INDEX}: holds no clause {beside}"));
        }
        counted.insert(beside);
        Ok(counted)
    }
}

/// The index `text` holds, or why it does not read: not TOML, a field
/// missing or one the index does not name, a section named twice or
/// holding no clause, or a clause that is no number or stands twice.
pub fn read_index(text: &str) -> Result<Index, String> {
    let written: Written =
        toml::from_str(text).map_err(|error| format!("{INDEX}: {}", error.message()))?;
    let mut names = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut sections = Vec::with_capacity(written.section.len());
    for section in written.section {
        if !names.insert(section.name.clone()) {
            return Err(format!(
                "{INDEX}: the section {} is named twice",
                section.name
            ));
        }
        if section.clauses.is_empty() {
            return Err(format!(
                "{INDEX}: the section {} holds no clause",
                section.name
            ));
        }
        let mut clauses = Vec::with_capacity(section.clauses.len());
        for value in &section.clauses {
            let clause = Clause::cited(value)
                .map_err(|why| format!("{INDEX}: section {}: {why}", section.name))?;
            if !seen.insert(clause.clone()) {
                return Err(format!("{INDEX}: the clause {clause} stands twice"));
            }
            clauses.push(clause);
        }
        sections.push(Section {
            name: section.name,
            clauses,
        });
    }
    Ok(Index {
        statement: written.statement,
        commit: written.commit,
        sections,
    })
}
