//! Taking lines: the seven steps a host takes each line of a run or an
//! intake in (13 to 23, 75, 76; C.1), fixed here so that every host takes
//! the same lines the same way (24; I8). The host hands in what is its
//! own: its inbox and its raw bank on this machine, as the two traits
//! below, and the machine's name, the taken-at time, the limit and how the
//! lines came in, as a [`Take`] (core.1, core.14).

use std::fmt;

use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

use crate::capture::{CameIn, Capture};
use crate::cut::{LeftOut, SMALLEST_LIMIT};
use crate::key::Key;
use crate::line::Nature;

/// Why an inbox or a bank refused, in its own words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal(pub String);

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refusal {}

/// The host's inbox, as taking needs it.
pub trait Inbox {
    /// Whether the inbox holds a capture under `key` (14).
    fn holds(&self, key: &Key) -> bool;

    /// Writes the capture and its own signal, when it has one, in one
    /// write (19; core.13), or refuses naming why.
    fn write(&mut self, taken: &Taken) -> Result<(), Refusal>;
}

/// The host's raw bank on this machine.
pub trait Bank {
    /// Banks a capture's raw files, whole and byte for byte, and the whole
    /// text of its own excerpt when that was cut, under the capture's name
    /// (17, 18, 21); returns the path on this machine where the capture's
    /// material now lies.
    fn bank(
        &mut self,
        key: &Key,
        files: &[RawFile],
        whole_excerpt: Option<&str>,
    ) -> Result<String, Refusal>;

    /// Removes what [`Bank::bank`] banked for `key`, whose capture was not
    /// written, so nothing a taken capture does not name is kept (18).
    fn discard(&mut self, key: &Key);
}

/// One raw file an offer names: its path relative to the raw place, as
/// offered, and its bytes as the adapter wrote them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// What one write hands the inbox: the capture, with its own signal when
/// its offer carried an own excerpt (19).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken {
    pub capture: Capture,
    pub own_signal: Option<OwnSignal>,
}

/// A capture's own signal, as `weir-signals` records it: every field as
/// offered but its excerpt, which is the text as cut (20).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnSignal {
    /// `<key>#1` (27).
    pub signal: String,
    pub kind: Nature,
    pub asserted_by: String,
    pub assertion: String,
    pub subjects: Vec<String>,
    pub argues_with: Vec<String>,
    pub position: String,
    /// The excerpt's text as cut, whole when within the limit.
    pub excerpt: String,
    /// What the cut left out, where it cut.
    pub cut: Option<LeftOut>,
}

/// What the host hands taking: the machine taking, by its short hostname
/// (17), when, the limit, and how the lines came in (22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Take {
    machine: String,
    taken_at: String,
    limit: usize,
    came_in: CameIn,
}

/// Why a [`Take`] could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TakeError {
    /// Below [`SMALLEST_LIMIT`], where the closing line might not fit.
    Limit(usize),
    /// Empty, or holding a `:` or a blank, which a pointer cannot name.
    Machine(String),
    /// A time RFC 3339 cannot write.
    TakenAt(String),
}

impl fmt::Display for TakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TakeError::Limit(limit) => write!(
                f,
                "the limit {limit} is below {SMALLEST_LIMIT} bytes, the closing line at its widest"
            ),
            TakeError::Machine(machine) => write!(
                f,
                "the machine `{machine}` is not a short hostname a pointer can name"
            ),
            TakeError::TakenAt(reason) => write!(f, "the taken-at time: {reason}"),
        }
    }
}

impl std::error::Error for TakeError {}

impl Take {
    /// A take on `machine` at `taken_at`, recorded in UTC, cutting own
    /// excerpts at `limit` bytes; a limit below [`SMALLEST_LIMIT`] is
    /// refused.
    pub fn new(
        machine: impl Into<String>,
        taken_at: OffsetDateTime,
        limit: usize,
        came_in: CameIn,
    ) -> Result<Take, TakeError> {
        if limit < SMALLEST_LIMIT {
            return Err(TakeError::Limit(limit));
        }
        let machine = machine.into();
        if machine.is_empty() || machine.chars().any(|c| c == ':' || c.is_whitespace()) {
            return Err(TakeError::Machine(machine));
        }
        let taken_at = taken_at
            .to_offset(UtcOffset::UTC)
            .format(&Rfc3339)
            .map_err(|error| TakeError::TakenAt(error.to_string()))?;
        Ok(Take {
            machine,
            taken_at,
            limit,
            came_in,
        })
    }

    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// In RFC 3339, in UTC.
    pub fn taken_at(&self) -> &str {
        &self.taken_at
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn came_in(&self) -> &CameIn {
        &self.came_in
    }
}
