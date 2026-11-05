//! A kind's identity, `<source>/<name>@<version>`, and a kind's name as a
//! reader is given it, its source or its version left out (88, C.4).

use std::fmt;
use std::str::FromStr;

use crate::name::{fault, NameRefusal, NAME};

/// The source of a repository's kinds (C.4; core.42).
pub const REPO: &str = "repo";

/// The source of a launch's kinds (C.4; core.42).
pub const LAUNCH: &str = "launch";

/// What a version is, as every reason that refuses one says it.
const VERSION: &str = "a whole number of at least 1, in decimal digits with no leading zero";

/// What an identity is, as a reason that finds a part missing says it.
const IDENTITY: &str = "an identity is `<source>/<name>@<version>`";

/// A kind's identity: its source, its name and its version, spelled
/// `<source>/<name>@<version>` (88, C.4). It spells back as the text it
/// was read from, and orders by source, then name, then version as a
/// number. A tool's own source is the tool's name as its reader gives it;
/// [`REPO`] and [`LAUNCH`] are a repository's and a launch's (core.42).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    source: String,
    name: String,
    version: u64,
}

impl Identity {
    /// Its source: a tool's name, [`REPO`], [`LAUNCH`], or the name of a
    /// source from outside.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The kind's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The kind's version, at least 1.
    pub fn version(&self) -> u64 {
        self.version
    }
}

impl FromStr for Identity {
    type Err = NameRefusal;

    /// An identity from `<source>/<name>@<version>`, or refused naming the
    /// part that is missing or fails.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let named = read(text, true).map_err(|reason| NameRefusal {
            text: text.to_owned(),
            reason: format!("is no identity: {reason}"),
        })?;
        Ok(named.identity().expect("a whole identity was read"))
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}@{}", self.source, self.name, self.version)
    }
}

/// A kind's name as a reader is given it: `<name>`, `<name>@<version>`,
/// `<source>/<name>` or `<source>/<name>@<version>`, the parts checked as
/// an identity's are (C.4 as amended at blueprints 361a599). It spells
/// back as it was written. What a name with a part left out names, and
/// which version it takes, is the reader's (58; core.32).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedKind {
    source: Option<String>,
    name: String,
    version: Option<u64>,
}

impl NamedKind {
    /// Its source, where given.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// The kind's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its version, where given; at least 1.
    pub fn version(&self) -> Option<u64> {
        self.version
    }

    /// The identity it names, when it gives its source and its version
    /// both; `None` when either is left out.
    pub fn identity(&self) -> Option<Identity> {
        Some(Identity {
            source: self.source.clone()?,
            name: self.name.clone(),
            version: self.version?,
        })
    }
}

impl FromStr for NamedKind {
    type Err = NameRefusal;

    /// A kind's name in any of its four forms, or refused naming the part
    /// that fails.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        read(text, false).map_err(|reason| NameRefusal {
            text: text.to_owned(),
            reason: format!("names no kind: {reason}"),
        })
    }
}

impl fmt::Display for NamedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = &self.source {
            write!(f, "{source}/")?;
        }
        f.write_str(&self.name)?;
        if let Some(version) = self.version {
            write!(f, "@{version}")?;
        }
        Ok(())
    }
}

/// The parts of `text`, the source before its first `/` and the version
/// after the first `@` that follows, each checked in that order; `whole`
/// when the source and the version must both be given. The error is why
/// the first part that fails does.
fn read(text: &str, whole: bool) -> Result<NamedKind, String> {
    let (source, rest) = match text.split_once('/') {
        Some((source, rest)) => (Some(source), rest),
        None => (None, text),
    };
    let (name, version) = match rest.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (rest, None),
    };
    match source {
        Some(source) => named("source", source)?,
        None if whole => return Err(format!("it names no source, and {IDENTITY}")),
        None => {}
    }
    named("name", name)?;
    let version = match version {
        Some(version) => Some(read_version(version)?),
        None if whole => return Err(format!("it names no version, and {IDENTITY}")),
        None => None,
    };
    Ok(NamedKind {
        source: source.map(str::to_owned),
        name: name.to_owned(),
        version,
    })
}

/// A part that must be a name, or why it is none.
fn named(part: &str, text: &str) -> Result<(), String> {
    match fault(text) {
        None => Ok(()),
        Some(fault) => Err(format!("{}, and a name is {NAME}", the(part, text, &fault))),
    }
}

/// A version's digits read as a number, or why they are no version: one
/// spelling for each version, so that a lock never holds one kind under
/// two (C.4 as amended at blueprints 361a599).
fn read_version(text: &str) -> Result<u64, String> {
    let fault = if text.is_empty() {
        "is empty".to_owned()
    } else if let Some(c) = text.chars().find(|c| !c.is_ascii_digit()) {
        format!("holds {c:?}")
    } else if text == "0" {
        "is 0".to_owned()
    } else if text.starts_with('0') {
        "has a leading zero".to_owned()
    } else {
        match text.parse() {
            Ok(version) => return Ok(version),
            Err(_) => format!("is more than {}", u64::MAX),
        }
    };
    Err(format!(
        "{}, and a version is {VERSION}",
        the("version", text, &fault)
    ))
}

/// A part named in a reason, by its text where it has one.
fn the(part: &str, text: &str, fault: &str) -> String {
    if text.is_empty() {
        format!("its {part} {fault}")
    } else {
        format!("its {part} `{text}` {fault}")
    }
}
