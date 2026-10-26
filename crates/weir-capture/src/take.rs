//! Taking lines: the seven steps a host takes each line of a run or an
//! intake in (13 to 23, 75, 76; C.1), fixed here so that every host takes
//! the same lines the same way (24; I8). The host hands in what is its
//! own: its inbox and its raw bank on this machine, as the two traits
//! below, and the machine's name, the taken-at time, the limit and how the
//! lines came in, as a [`Take`] (core.1, core.14).

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

use crate::capture::{CameIn, Capture};
use crate::cut::{cut, LeftOut, SMALLEST_LIMIT};
use crate::key::Key;
use crate::line::{read_line, Failure, Line, Nature, Offer};

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

/// One line refused: its number, from 1, the key it offered where it had
/// one, and why (15).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Refused {
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub reason: String,
}

/// What taking reports of a run or an intake (23, 76), which the host
/// keeps or prints as JSON (C.2: JSON only in flight).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The keys taken, in the order of their lines.
    pub taken: Vec<Key>,
    pub held: u64,
    pub refused: Vec<Refused>,
    /// How many events the completion said the listing held, where a
    /// completion was read.
    pub listed: Option<u64>,
    /// Each failure the completion named.
    pub failed: Vec<Failure>,
    /// The last cursor a run's lines held. Only a run keeps a cursor: an
    /// intake's or an import's lines keep none.
    #[serde(skip)]
    pub last_cursor: Option<String>,
    /// Set by [`Report::finish`].
    pub complete: bool,
    /// The cursor kept, set by [`Report::finish`].
    pub cursor: Option<String>,
}

impl Report {
    /// The report of a run that has ended: complete only when a completion
    /// was read and the adapter exited reporting success, and the last
    /// cursor read kept only then (9, 23; core.15). An intake, where no
    /// adapter ran, finishes with `true`.
    pub fn finish(mut self, exited_ok: bool) -> Report {
        self.complete = exited_ok && self.listed.is_some();
        self.cursor = if self.complete {
            self.last_cursor.clone()
        } else {
            None
        };
        self
    }
}

/// Takes `lines`, a run's or an intake's, in the seven steps of C.1: each
/// line is checked, and a malformed line or an offer that fails the
/// contract is refused, named and passed over (13, 15); a key the inbox
/// holds, or that an earlier line of `lines` was taken under, is counted
/// held, and nothing is banked or written for it (14); every other offer's
/// raw files are read from `raw_place` and banked whole, with its excerpt's
/// whole text when cut, before its capture is written, pointing at what
/// was banked (17, 18, 21). The report holds what the lines took, held and
/// refused, their completion and their last cursor, for the host to
/// [`Report::finish`] once the adapter has exited (23).
pub fn take<L: AsRef<str>>(
    lines: impl IntoIterator<Item = L>,
    raw_place: Option<&Path>,
    how: &Take,
    inbox: &mut impl Inbox,
    bank: &mut impl Bank,
) -> Report {
    let mut report = Report::default();
    // The inbox may answer `holds` from a tip read before the take; a key
    // taken here is held from here on all the same.
    let mut taken_here: HashSet<Key> = HashSet::new();
    for (number, line) in (1..).zip(lines) {
        match read_line(line.as_ref()) {
            Err(malformed) => report.refused.push(Refused {
                line: number,
                key: malformed.key.clone(),
                reason: malformed.to_string(),
            }),
            Ok(Line::Cursor(cursor)) => {
                if matches!(how.came_in, CameIn::Run(_)) {
                    report.last_cursor = Some(cursor);
                }
            }
            Ok(Line::Done(done)) => {
                report.listed = Some(done.listed);
                report.failed = done.failed;
            }
            Ok(Line::Offer(offer)) => {
                if taken_here.contains(&offer.key) || inbox.holds(&offer.key) {
                    report.held += 1;
                    continue;
                }
                let key = offer.key.clone();
                match take_offer(offer, raw_place, how, inbox, bank) {
                    Ok(()) => {
                        taken_here.insert(key.clone());
                        report.taken.push(key);
                    }
                    Err(reason) => report.refused.push(Refused {
                        line: number,
                        key: Some(key.to_string()),
                        reason,
                    }),
                }
            }
        }
    }
    report
}

/// Banks and writes one offer whose key the inbox does not hold, or says
/// why not: steps 3 to 6 of C.1. An own excerpt, cut to the limit, becomes
/// the capture's one signal, written with it in one write, and the capture
/// is not unread; every other capture is written unread (19, 20). When the
/// inbox refuses the write, what was banked for it is discarded (18).
fn take_offer(
    offer: Offer,
    raw_place: Option<&Path>,
    how: &Take,
    inbox: &mut impl Inbox,
    bank: &mut impl Bank,
) -> Result<(), String> {
    let files = read_raw(&offer.raw, raw_place)?;
    let excerpt = offer.excerpt.map(|excerpt| {
        let cut = cut(&excerpt.text, how.limit);
        (excerpt, cut)
    });
    let whole_excerpt = excerpt
        .as_ref()
        .filter(|(_, cut)| cut.left_out.is_some())
        .map(|(excerpt, _)| excerpt.text.as_str());
    let pointer = if files.is_empty() && whole_excerpt.is_none() {
        None
    } else {
        match bank.bank(&offer.key, &files, whole_excerpt) {
            Ok(path) => Some(format!("{}:{path}", how.machine)),
            Err(refusal) => {
                // Whatever it banked before refusing is not kept (18).
                bank.discard(&offer.key);
                return Err(format!("the bank refused the raw material: {refusal}"));
            }
        }
    };
    let own_signal = excerpt.map(|(excerpt, cut)| OwnSignal {
        signal: format!("{}#1", offer.key),
        kind: excerpt.kind,
        asserted_by: excerpt.asserted_by,
        assertion: excerpt.assertion,
        subjects: excerpt.subjects,
        argues_with: excerpt.argues_with,
        position: excerpt.position,
        excerpt: cut.text,
        cut: cut.left_out,
    });
    let capture = Capture {
        key: offer.key,
        source: offer.source,
        event_at: offer.event_at,
        title: offer.title,
        url: offer.url,
        pointer,
        came_in: how.came_in.clone(),
        machine: how.machine.clone(),
        taken_at: how.taken_at.clone(),
        unread: own_signal.is_none(),
    };
    let taken = Taken {
        capture,
        own_signal,
    };
    inbox.write(&taken).map_err(|refusal| {
        if taken.capture.pointer.is_some() {
            bank.discard(&taken.capture.key);
        }
        format!("the inbox refused the capture: {refusal}")
    })
}

/// The raw files `raw` names, read whole from `raw_place`; a file that is
/// missing, cannot be read or resolves outside the raw place refuses the
/// offer, as does any raw material named with no raw place (6, 18).
fn read_raw(raw: &[String], raw_place: Option<&Path>) -> Result<Vec<RawFile>, String> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let named = raw
        .iter()
        .map(|path| format!("`{path}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let Some(raw_place) = raw_place else {
        return Err(format!(
            "the offer names raw material, {named}, and no raw place was given"
        ));
    };
    let place = raw_place.canonicalize().map_err(|error| {
        format!(
            "the offer names raw material, {named}, and the raw place {} cannot be read: {error}",
            raw_place.display()
        )
    })?;
    raw.iter()
        .map(|path| {
            let resolved = place
                .join(path)
                .canonicalize()
                .map_err(|error| format!("raw `{path}` cannot be read: {error}"))?;
            if !resolved.starts_with(&place) {
                return Err(format!("raw `{path}` resolves outside the raw place"));
            }
            let bytes = fs::read(&resolved)
                .map_err(|error| format!("raw `{path}` cannot be read: {error}"))?;
            Ok(RawFile {
                path: path.clone(),
                bytes,
            })
        })
        .collect()
}
