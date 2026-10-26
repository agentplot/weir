//! The inbox read whole from the files of one tip (43; core.23): every
//! record where the layout places it, in a format and major this crate
//! reads, and agreeing with the records beside it. Every operation answers
//! from one such read (46).

use std::collections::BTreeMap;
use std::fmt;

use weir_capture::rec::{self, RecError, Record};
use weir_capture::{Capture, Key};

use crate::format::Format;
use crate::layout::Place;
use crate::signal::{Signal, SignalId, EMPTY_READING};

/// An inbox as one tip holds it. The name keeps it apart from
/// `weir_capture::Inbox`, the trait a host implements for `take`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxAt {
    /// Every file under `inbox/`, by path, as the tip holds it.
    files: BTreeMap<String, String>,
    captures: BTreeMap<Key, Held>,
}

/// One capture the inbox holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub capture: Capture,
    /// Its own signal or its reading, from its `signals.rec`; none while
    /// it is unread (19, 28).
    pub signals: Option<Vec<Signal>>,
}

/// Why an inbox could not be read: the path, and where there is one, the
/// record, counting from 1, and its field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxError {
    pub path: String,
    pub record: Option<usize>,
    pub field: Option<String>,
    pub reason: String,
}

impl InboxError {
    pub(crate) fn at(path: impl Into<String>, reason: impl Into<String>) -> InboxError {
        InboxError {
            path: path.into(),
            record: None,
            field: None,
            reason: reason.into(),
        }
    }

    pub(crate) fn field(
        path: impl Into<String>,
        record: Option<usize>,
        error: RecError,
    ) -> InboxError {
        InboxError {
            path: path.into(),
            record,
            field: error.field,
            reason: error.reason,
        }
    }
}

impl fmt::Display for InboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.path)?;
        if let Some(record) = self.record {
            write!(f, ", record {record}")?;
        }
        f.write_str(": ")?;
        if let Some(field) = &self.field {
            write!(f, "`{field}` ")?;
        }
        f.write_str(&self.reason)
    }
}

impl std::error::Error for InboxError {}

/// The records of the file at `path`, each naming a format of `lies`.
fn records(path: &str, text: &str, lies: &[Format]) -> Result<Vec<Record>, InboxError> {
    let records = rec::read(text).map_err(|error| InboxError::field(path, None, error))?;
    for (n, record) in (1..).zip(&records) {
        Format::of_record(record, lies).map_err(|error| InboxError::field(path, Some(n), error))?;
    }
    Ok(records)
}

impl InboxAt {
    /// The inbox the files of one tip hold, each as `(path, text)` from the
    /// tip's root: those under `inbox/` are read, and every other passed
    /// over. A file the layout does not place, a record out of its place,
    /// in no format or in one this crate does not read, or at odds with the
    /// records beside it, refuses the whole inbox, naming the path.
    pub fn read<P, T>(files: impl IntoIterator<Item = (P, T)>) -> Result<InboxAt, InboxError>
    where
        P: AsRef<str>,
        T: AsRef<str>,
    {
        let mut texts = BTreeMap::new();
        let mut places = BTreeMap::new();
        for (path, text) in files {
            let path = path.as_ref();
            let Some(place) = Place::of(path) else {
                continue;
            };
            let place = place.map_err(|reason| InboxError::at(path, reason))?;
            if texts
                .insert(path.to_string(), text.as_ref().to_string())
                .is_some()
            {
                return Err(InboxError::at(path, "is given twice"));
            }
            places.insert(place, path.to_string());
        }
        let mut inbox = InboxAt {
            files: texts,
            captures: BTreeMap::new(),
        };
        inbox.read_places(&places)?;
        Ok(inbox)
    }

    fn read_places(&mut self, places: &BTreeMap<Place, String>) -> Result<(), InboxError> {
        for (place, path) in places {
            let text = &self.files[path];
            match place {
                Place::Capture(key) => {
                    let capture = read_capture(path, text, key)?;
                    self.captures.insert(
                        key.clone(),
                        Held {
                            capture,
                            signals: None,
                        },
                    );
                }
                // Every capture's place orders before every signals' place,
                // so the capture beside these signals is read by now.
                Place::Signals(key) => {
                    let Some(held) = self.captures.get_mut(key) else {
                        return Err(InboxError::at(
                            path,
                            "lies beside no `capture.rec`: a capture's signals lie beside the capture",
                        ));
                    };
                    held.signals = Some(read_signals(path, text, key)?);
                }
                Place::Moves => {
                    records(path, text, &[Format::Move])?;
                }
                Place::Batch(_) => {
                    records(path, text, &[Format::Batch, Format::Move])?;
                }
                Place::Item(_) => {
                    records(path, text, &[Format::Item])?;
                }
                Place::Brief(_) => {
                    Format::read_brief(text).map_err(|reason| InboxError::at(path, reason))?;
                }
                Place::Sends(_) => {
                    records(path, text, &[Format::Send])?;
                }
            }
        }
        Ok(())
    }

    /// Whether the inbox holds a capture under `key` (14).
    pub fn holds(&self, key: &Key) -> bool {
        self.captures.contains_key(key)
    }

    /// The capture the inbox holds under `key`.
    pub fn capture(&self, key: &Key) -> Option<&Capture> {
        self.captures.get(key).map(|held| &held.capture)
    }

    /// The capture's own signal or its reading, or none while it is
    /// unread.
    pub fn signals(&self, key: &Key) -> Option<&[Signal]> {
        self.captures.get(key)?.signals.as_deref()
    }

    /// The signal `id` names, where the inbox holds it.
    pub fn signal(&self, id: &SignalId) -> Option<&Signal> {
        let n = usize::try_from(id.n).ok()?.checked_sub(1)?;
        self.signals(&id.key)?.get(n)
    }

    /// Every key the inbox holds, in order.
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.captures.keys()
    }

    /// Every file under `inbox/`, by path, as the tip holds it.
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
}

/// The one `weir-capture/1` record of the `capture.rec` at `path`, which
/// names the key its folder is (42).
fn read_capture(path: &str, text: &str, key: &Key) -> Result<Capture, InboxError> {
    let records = records(path, text, &[Format::Capture])?;
    let [record] = records.as_slice() else {
        return Err(InboxError::at(
            path,
            format!(
                "holds {} records, and a capture's record is one",
                records.len()
            ),
        ));
    };
    let capture =
        Capture::from_record(record).map_err(|error| InboxError::field(path, Some(1), error))?;
    if &capture.key != key {
        return Err(InboxError::field(
            path,
            Some(1),
            RecError::field(
                "key",
                format!(
                    "is `{}`, and the capture lies in the folder of `{key}`",
                    capture.key
                ),
            ),
        ));
    }
    Ok(capture)
}

/// The signals of the `signals.rec` at `path`, beside the capture of `key`:
/// each naming that key, numbered from 1 in order (27), or none, for a
/// reading of no signals, where the file names its format in its one line.
fn read_signals(path: &str, text: &str, key: &Key) -> Result<Vec<Signal>, InboxError> {
    let records = records(path, text, &[Format::Signal])?;
    if records.is_empty() {
        if !text.lines().any(|line| line == EMPTY_READING) {
            return Err(InboxError::at(
                path,
                format!(
                    "holds no record, and a reading of no signals is the one line `{EMPTY_READING}`"
                ),
            ));
        }
        return Ok(Vec::new());
    }
    let mut signals = Vec::with_capacity(records.len());
    for (n, record) in (1..).zip(&records) {
        let signal =
            Signal::from_record(record).map_err(|error| InboxError::field(path, Some(n), error))?;
        let id = &signal.signal;
        if &id.key != key {
            return Err(InboxError::field(
                path,
                Some(n),
                RecError::field(
                    "signal",
                    format!("is `{id}`, which names another capture than `{key}`"),
                ),
            ));
        }
        if id.n != n as u64 {
            return Err(InboxError::field(
                path,
                Some(n),
                RecError::field(
                    "signal",
                    format!(
                        "is `{id}` where `{key}#{n}` stands: a capture's signals are numbered from 1 in reading order"
                    ),
                ),
            ));
        }
        signals.push(signal);
    }
    Ok(signals)
}
