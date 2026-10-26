//! The import operation (75a): captures read before their inbox existed,
//! carried into it with the pointer, signals and moves each was read with
//! (37 to 39, 63). Each is held by key like any other capture, so a body
//! imported twice writes nothing the second time (38); nothing is banked,
//! no pointer of the host's own is written and no excerpt is looked for
//! (37); a move not of the six, or without its consequence, is not taken,
//! and its signal arrives unmoved (39). The import line itself is read in
//! `weir-capture`; what it carries is handed here as [`ImportedCapture`].

use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};
use weir_capture::rec::{self, Record};
use weir_capture::{CameIn, Capture, Key};

use crate::inbox::{Checked, Effect, Held, InboxAt, ItemAt};
use crate::layout::Place;
use crate::moves::{check_time, NewMove};
use crate::signal::{signals_rec, ReadSignal, Signal, SignalId};
use crate::write::{joined_item, NotTaken, Refused, Write};

/// What one import line carries (C.1): the capture as an offer would,
/// without raw material or an own excerpt, the pointer it was held under,
/// how it came in, and its signals and moves as read. The key, source and
/// event time are the line's text, so the import refuses a capture that
/// names none, whatever built the value (75a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedCapture {
    pub key: String,
    pub source: String,
    pub event_at: String,
    pub title: Option<String>,
    pub url: Option<String>,
    /// As the body gave it, or none (37).
    pub pointer: Option<String>,
    /// `import:<body>` (22).
    pub came_in: CameIn,
    /// In the body's order, numbered from 1.
    pub signals: Vec<ImportedSignal>,
    pub moves: Vec<ImportedMove>,
}

/// One signal of an imported capture, with its number in the body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedSignal {
    pub n: u64,
    pub signal: ReadSignal,
}

/// One move of an imported capture, naming its signal by number, its word
/// kept as it stands in the body (39).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedMove {
    pub signal: u64,
    pub word: String,
    pub target: String,
    pub by: String,
    pub at: String,
}

/// What an import formed: the write, none when it writes nothing, and its
/// report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    pub write: Option<Write>,
    pub report: ImportReport,
}

/// What an import reports (75a): the captures written, in the order of
/// their lines, the count held, each refusal of a capture refused, and each
/// move not taken, each with why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub written: Vec<Key>,
    pub held: u64,
    pub refused: Vec<CaptureRefused>,
    pub not_moved: Vec<MoveNotTaken>,
}

/// Why an imported capture was refused whole, its signals and moves with
/// it: the key it names, as the line gave it, the signal where the refusal
/// is one of its signal's, the field and why. A capture refused for its
/// signals is named once for each field of each signal not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureRefused {
    pub key: String,
    pub signal: Option<SignalId>,
    pub field: String,
    pub why: String,
}

/// An imported move not taken: its signal, which arrives unmoved, its word
/// as the body gave it, the field where there is one, and why (39).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveNotTaken {
    pub signal: SignalId,
    pub word: String,
    pub field: Option<String>,
    pub why: String,
}

impl InboxAt {
    /// The import of `captures`, in order, by the machine `machine` at
    /// `imported_at`, as one write checked at this tip (45, 46), with its
    /// report (75a).
    ///
    /// A capture that names no key or one the key rule refuses, no source,
    /// an event time not in RFC 3339 with its offset, or a way in other
    /// than an import, is refused whole and reported with the field (4,
    /// 13). A key the inbox holds, or that an earlier capture of the
    /// import took, is counted held and passed over with its signals and
    /// moves (14, 38). A capture whose signals do not number from 1 in
    /// order, or whose signal lacks a field or names a nature not of the
    /// five, is refused whole (25, 28). Every other capture is written with
    /// its pointer as given, the machine and time handed in, unread when it
    /// has no signals, and its signals as its reading, no excerpt looked
    /// for (37). Each of its moves is checked as [`InboxAt::make_move`]
    /// checks one, against the inbox as the import has grown it; one not
    /// taken is reported, its signal left unmoved, and an imported join
    /// makes or grows a proposed item with no brief (39).
    pub fn import(
        &self,
        captures: &[ImportedCapture],
        machine: &str,
        imported_at: OffsetDateTime,
    ) -> Result<Imported, Refused> {
        // The machine and the time are written as take writes them (17,
        // 22): a short hostname a pointer can name, and the time in UTC.
        if machine.is_empty() || machine.chars().any(|c| c == ':' || c.is_whitespace()) {
            return Err(NotTaken::field(
                machine,
                "machine",
                "is not a short hostname a pointer can name",
            )
            .into());
        }
        let imported_at = imported_at
            .to_offset(UtcOffset::UTC)
            .format(&Rfc3339)
            .map_err(|error| NotTaken::field(machine, "taken_at", error.to_string()))?;
        let mut grown = self.clone();
        let mut report = ImportReport::default();
        let mut write = Write::new();
        let mut moves: Vec<Record> = Vec::new();
        for imported in captures {
            let key = match imported.check() {
                Ok(key) => key,
                Err(refused) => {
                    report.refused.push(refused);
                    continue;
                }
            };
            if grown.holds(&key) {
                report.held += 1;
                continue;
            }
            let signals = match imported.read_signals(&key) {
                Ok(signals) => signals,
                Err(refused) => {
                    report.refused.extend(refused);
                    continue;
                }
            };
            let capture = Capture {
                key: key.clone(),
                source: imported.source.clone(),
                event_at: imported.event_at.clone(),
                title: imported.title.clone(),
                url: imported.url.clone(),
                pointer: imported.pointer.clone(),
                came_in: imported.came_in.clone(),
                machine: machine.to_string(),
                taken_at: imported_at.clone(),
                unread: signals.is_empty(),
            };
            write.put(Place::Capture(key.clone()).path(), capture.to_rec());
            if !signals.is_empty() {
                write.put(Place::Signals(key.clone()).path(), signals_rec(&signals));
            }
            grown.captures.insert(
                key.clone(),
                Held {
                    capture,
                    signals: (!signals.is_empty()).then_some(signals),
                },
            );
            report.written.push(key.clone());
            for imported_move in &imported.moves {
                let new = NewMove {
                    signal: SignalId::new(key.clone(), imported_move.signal),
                    word: imported_move.word.clone(),
                    target: imported_move.target.clone(),
                    by: imported_move.by.clone(),
                    at: imported_move.at.clone(),
                    replaces: None,
                };
                let not_moved = |field: Option<String>, why: String| MoveNotTaken {
                    signal: new.signal.clone(),
                    word: new.word.clone(),
                    field,
                    why,
                };
                let taken = match new.to_move(None) {
                    Ok(taken) => taken,
                    Err((field, why)) => {
                        report.not_moved.push(not_moved(Some(field), why));
                        continue;
                    }
                };
                match grown.check_move(&taken, Checked::Formed) {
                    Err(not) => {
                        report
                            .not_moved
                            .push(not_moved(not.field.map(str::to_string), not.why));
                        continue;
                    }
                    Ok(Effect::MakesItem) => {
                        let item = joined_item(&taken);
                        write.put(
                            Place::Item(item.item.clone()).path(),
                            rec::write(&[item.to_record()]),
                        );
                        grown.items.insert(
                            item.item.clone(),
                            ItemAt {
                                item,
                                brief: None,
                                sends: Vec::new(),
                            },
                        );
                    }
                    Ok(Effect::Stands) => {}
                }
                moves.push(taken.to_record());
                grown.push_move(taken);
            }
        }
        if !moves.is_empty() {
            let path = Place::Moves.path();
            write.put(&path, self.appended(&path, &moves));
        }
        if write.is_empty() {
            return Ok(Imported {
                write: None,
                report,
            });
        }
        self.check(&write)?;
        Ok(Imported {
            write: Some(write),
            report,
        })
    }
}

impl ImportedCapture {
    /// Its key, or why it is refused whole before its key is looked up:
    /// no key or one the rule refuses, no source, an event time not in RFC
    /// 3339 with its offset, or a way in other than an import (4, 13, 75a).
    fn check(&self) -> Result<Key, CaptureRefused> {
        let refused = |field: &str, why: String| CaptureRefused {
            key: self.key.clone(),
            signal: None,
            field: field.to_string(),
            why,
        };
        let key = Key::parse(&self.key).map_err(|error| {
            if self.key.is_empty() {
                refused("key", "is missing: the line's capture names no key".into())
            } else {
                refused("key", format!("is `{}`: {error}", self.key))
            }
        })?;
        if self.source.is_empty() {
            return Err(refused(
                "source",
                "is missing: the line's capture names no source".into(),
            ));
        }
        check_time(&self.event_at).map_err(|why| refused("event_at", why))?;
        if !matches!(self.came_in, CameIn::Import(_)) {
            return Err(refused(
                "came_in",
                format!(
                    "is `{}`, and an imported capture came in by an import, `import:<body>`",
                    self.came_in
                ),
            ));
        }
        Ok(key)
    }

    /// Its signals as its reading, numbered from 1 in the body's order, or
    /// each signal and field that refuses the capture whole (25, 28). No
    /// excerpt is looked for in raw material (26, 37).
    fn read_signals(&self, key: &Key) -> Result<Vec<Signal>, Vec<CaptureRefused>> {
        let mut signals = Vec::with_capacity(self.signals.len());
        let mut refused = Vec::new();
        for (n, imported) in (1..).zip(&self.signals) {
            let id = SignalId::new(key.clone(), imported.n);
            let refusal = |field: &str, why: String| CaptureRefused {
                key: self.key.clone(),
                signal: Some(id.clone()),
                field: field.to_string(),
                why,
            };
            if imported.n != n {
                refused.push(refusal(
                    "n",
                    format!(
                        "is {}, where {n} stands: an import's signals number from 1 in the body's order",
                        imported.n
                    ),
                ));
                continue;
            }
            match imported.signal.to_signal(id.clone()) {
                Ok(signal) => signals.push(signal),
                Err(failures) => refused.extend(
                    failures
                        .into_iter()
                        .map(|(field, why)| refusal(&field, why)),
                ),
            }
        }
        if refused.is_empty() {
            Ok(signals)
        } else {
            Err(refused)
        }
    }
}
