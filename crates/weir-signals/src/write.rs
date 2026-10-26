//! A write, and the operations that form one: each a method of [`InboxAt`]
//! taking what its caller hands, forming the full new text of every file
//! it adds or replaces from the inbox read at one tip (47), and checking
//! the write there (46). The write protocol forms it again on a new tip by
//! calling the same operation on the inbox read there, so every check is
//! made again (44; core.26).

use std::collections::BTreeMap;
use std::fmt;

use weir_capture::rec::{self, Record};
use weir_capture::{CameIn, Key, RawFile, Taken};

use crate::inbox::{Checked, Effect, InboxAt, InboxError, NotMoved};
use crate::item::{Item, MadeBy};
use crate::layout::Place;
use crate::moves::{Move, NewMove};
use crate::signal::{signals_rec, ReadSignal, Signal, SignalId};

/// The full new text of each file a write adds or replaces, by path from
/// the tip's root: an append is the tip's text with records added after it
/// (47).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Write {
    pub files: BTreeMap<String, String>,
}

impl Write {
    pub fn new() -> Write {
        Write::default()
    }

    /// Gives the file at `path` the text `text`.
    pub fn put(&mut self, path: impl Into<String>, text: impl Into<String>) {
        self.files.insert(path.into(), text.into());
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// One thing a write did not take: what it names (a path, a key, a signal,
/// a move, a batch or an item), the field where there is one, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotTaken {
    pub what: String,
    pub field: Option<String>,
    pub why: String,
}

impl NotTaken {
    pub(crate) fn new(what: impl fmt::Display, why: impl Into<String>) -> NotTaken {
        NotTaken {
            what: what.to_string(),
            field: None,
            why: why.into(),
        }
    }

    pub(crate) fn field(
        what: impl fmt::Display,
        field: impl Into<String>,
        why: impl Into<String>,
    ) -> NotTaken {
        NotTaken {
            what: what.to_string(),
            field: Some(field.into()),
            why: why.into(),
        }
    }
}

impl From<InboxError> for NotTaken {
    fn from(error: InboxError) -> NotTaken {
        let why = match error.record {
            Some(record) => format!("record {record}: {}", error.reason),
            None => error.reason,
        };
        NotTaken {
            what: error.path,
            field: error.field,
            why,
        }
    }
}

impl fmt::Display for NotTaken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`: ", self.what)?;
        if let Some(field) = &self.field {
            write!(f, "`{field}` ")?;
        }
        f.write_str(&self.why)
    }
}

/// A write refused whole, naming each thing not taken and why; nothing of
/// it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused(pub Vec<NotTaken>);

impl Refused {
    pub(crate) fn one(not_taken: NotTaken) -> Refused {
        Refused(vec![not_taken])
    }
}

impl From<InboxError> for Refused {
    fn from(error: InboxError) -> Refused {
        Refused::one(error.into())
    }
}

impl From<NotTaken> for Refused {
    fn from(not_taken: NotTaken) -> Refused {
        Refused::one(not_taken)
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let each: Vec<String> = self.0.iter().map(NotTaken::to_string).collect();
        f.write_str(&each.join("; "))
    }
}

impl std::error::Error for Refused {}

impl NotMoved {
    /// The refusal of the move of `signal` this names.
    pub(crate) fn of(self, signal: &SignalId) -> NotTaken {
        NotTaken {
            what: signal.to_string(),
            field: self.field.map(str::to_string),
            why: self.why,
        }
    }
}

/// The item a join naming a name no item has had makes: proposed, named by
/// the move's maker at the move's time (34).
pub(crate) fn joined_item(join: &Move) -> Item {
    Item {
        item: join.target.clone(),
        made_by: MadeBy::Join,
        named_by: join.by.clone(),
        named_at: join.at.clone(),
    }
}

impl InboxAt {
    /// The new text of the file at `path` with `records` appended after the
    /// tip's: a blank line, then the records (41, 47).
    pub(crate) fn appended(&self, path: &str, records: &[Record]) -> String {
        let added = rec::write(records);
        match self.files.get(path) {
            None => added,
            Some(text) if text.is_empty() => added,
            Some(text) => {
                let mut whole = text.clone();
                if !whole.ends_with('\n') {
                    whole.push('\n');
                }
                whole.push('\n');
                whole.push_str(&added);
                whole
            }
        }
    }

    /// The inbox `write` would leave, laid over this tip's files and read
    /// whole by every rule of [`InboxAt::read`], or the write refused
    /// naming the path. Every operation ends here, and a caller may hand in
    /// a write formed elsewhere (46; core.24).
    pub fn check(&self, write: &Write) -> Result<InboxAt, Refused> {
        for path in write.files.keys() {
            if Place::of(path).is_none() {
                return Err(NotTaken::new(
                    path,
                    "lies outside `inbox/`, and a write changes the inbox alone",
                )
                .into());
            }
        }
        let mut files = self.files().clone();
        files.extend(write.files.clone());
        Ok(InboxAt::read(&files)?)
    }

    /// The write of a capture the capture crate's `take` hands over:
    /// `capture.rec` and, with its own signal, a `signals.rec` holding
    /// `<key>#1`, in one write (19; core.13); refused when the key is held,
    /// writing nothing (14).
    pub fn take(&self, taken: &Taken) -> Result<Write, Refused> {
        let capture = &taken.capture;
        let key = &capture.key;
        if self.holds(key) {
            return Err(NotTaken::new(
                key,
                "is held: the inbox holds a capture under it, and nothing is written",
            )
            .into());
        }
        if let CameIn::Import(_) = capture.came_in {
            return Err(NotTaken::field(
                key,
                "came_in",
                format!(
                    "is `{}`: an import's captures are written by the import operation alone",
                    capture.came_in
                ),
            )
            .into());
        }
        let mut write = Write::new();
        write.put(Place::Capture(key.clone()).path(), capture.to_rec());
        match &taken.own_signal {
            Some(own) => {
                if capture.unread {
                    return Err(NotTaken::field(
                        key,
                        "unread",
                        "is yes, and a capture with its own signal is never unread",
                    )
                    .into());
                }
                let signal = Signal::own(own).map_err(|why| NotTaken::field(key, "signal", why))?;
                let first = SignalId::new(key.clone(), 1);
                if signal.signal != first {
                    return Err(NotTaken::field(
                        key,
                        "signal",
                        format!(
                            "is `{}`, and a capture's own signal is `{first}`",
                            signal.signal
                        ),
                    )
                    .into());
                }
                write.put(Place::Signals(key.clone()).path(), signals_rec(&[signal]));
            }
            None => {
                if !capture.unread {
                    return Err(NotTaken::field(
                        key,
                        "unread",
                        "is no, and a capture with no own signal is unread when written",
                    )
                    .into());
                }
            }
        }
        self.check(&write)?;
        Ok(write)
    }

    /// The write of a reader's reading of the capture under `key`:
    /// `signals.rec`, the signals numbered from 1 in the order handed (27),
    /// or, for a reading of none, the one line that marks the capture read
    /// (28). Refused when the capture is not held, or has its own signal or
    /// a reading already (28; core.17); refused whole, naming each signal
    /// not taken and why, when any lacks a field, names a nature not of the
    /// five (25) or has an excerpt whose bytes stand in none of the
    /// capture's raw files the caller hands in, read from its bank (26;
    /// core.16).
    pub fn read_capture(
        &self,
        key: &Key,
        signals: &[ReadSignal],
        raw: &[RawFile],
    ) -> Result<Write, Refused> {
        let Some(held) = self.captures.get(key) else {
            return Err(NotTaken::new(key, "is no capture the inbox holds").into());
        };
        if held.signals.is_some() {
            let why = if held.has_own_signal() {
                "has its own signal, and a capture whose offer carried an own excerpt takes no reading"
            } else {
                "has a reading, and a capture has at most one"
            };
            return Err(NotTaken::new(key, why).into());
        }
        let mut taken = Vec::with_capacity(signals.len());
        let mut not_taken = Vec::new();
        for (n, read) in (1..).zip(signals) {
            let id = SignalId::new(key.clone(), n);
            match read.to_signal(id.clone()) {
                Err(failures) => not_taken.extend(
                    failures
                        .into_iter()
                        .map(|(field, why)| NotTaken::field(&id, field, why)),
                ),
                Ok(signal) if !stands_in(signal.excerpt.as_bytes(), raw) => {
                    not_taken.push(NotTaken::field(
                        &id,
                        "excerpt",
                        "stands in none of the capture's raw files, byte for byte",
                    ))
                }
                Ok(signal) => taken.push(signal),
            }
        }
        if !not_taken.is_empty() {
            return Err(Refused(not_taken));
        }
        let mut write = Write::new();
        write.put(Place::Signals(key.clone()).path(), signals_rec(&taken));
        self.check(&write)?;
        Ok(write)
    }

    /// The write of one move: a record appended to `moves.rec`, and, for a
    /// join naming a name no item has had, that item's `item.rec` (29 to
    /// 31; core.18). Refused naming the move's signal when its word is not
    /// one of the six, its consequence is missing or no item's name where
    /// it is one, its signal is not held, the signal has a standing move it
    /// does not replace by number, or it joins a sent item or attaches to
    /// one not sent.
    pub fn make_move(&self, new: &NewMove) -> Result<Write, Refused> {
        let taken = new
            .to_move(None)
            .map_err(|(field, why)| NotTaken::field(&new.signal, field, why))?;
        let effect = self
            .check_move(&taken, Checked::Formed)
            .map_err(|not| not.of(&new.signal))?;
        let mut write = Write::new();
        let moves = Place::Moves.path();
        write.put(&moves, self.appended(&moves, &[taken.to_record()]));
        if effect == Effect::MakesItem {
            write.put(
                Place::Item(taken.target.clone()).path(),
                rec::write(&[joined_item(&taken).to_record()]),
            );
        }
        self.check(&write)?;
        Ok(write)
    }
}

/// Whether `excerpt` stands, byte for byte, in one of `raw` (26). An empty
/// excerpt stands nowhere: it is a field lacking.
fn stands_in(excerpt: &[u8], raw: &[RawFile]) -> bool {
    !excerpt.is_empty()
        && raw.iter().any(|file| {
            file.bytes
                .windows(excerpt.len())
                .any(|window| window == excerpt)
        })
}
