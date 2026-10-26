//! The inbox read whole from the files of one tip (43; core.23): every
//! record where the layout places it, in a format and major this crate
//! reads, and agreeing with the records beside it. Every operation answers
//! from one such read (46).

use std::collections::BTreeMap;
use std::fmt;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::rec::{self, RecError, Record};
use weir_capture::{CameIn, Capture, Key};

use crate::format::Format;
use crate::item::{Item, SendRecord};
use crate::layout::Place;
use crate::moves::{Batch, Decision, Move, Proposal, Word};
use crate::signal::{Signal, SignalId, EMPTY_READING};

/// An inbox as one tip holds it. The name keeps it apart from
/// `weir_capture::Inbox`, the trait a host implements for `take`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxAt {
    /// Every file under `inbox/`, by path, as the tip holds it.
    pub(crate) files: BTreeMap<String, String>,
    pub(crate) captures: BTreeMap<Key, Held>,
    /// `moves.rec`'s moves, in order.
    pub(crate) moves: Vec<Move>,
    /// Each moved signal's moves, as places in `moves`, in order: the last
    /// is its standing move (29).
    pub(crate) by_signal: BTreeMap<SignalId, Vec<usize>>,
    pub(crate) items: BTreeMap<String, ItemAt>,
    pub(crate) batches: BTreeMap<String, Batch>,
}

/// One capture the inbox holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Held {
    pub capture: Capture,
    /// Its own signal or its reading, from its `signals.rec`; none while
    /// it is unread (19, 28).
    pub signals: Option<Vec<Signal>>,
}

/// A signal with no standing move, and its capture's event time (33).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unmoved<'a> {
    pub signal: &'a Signal,
    pub event_at: &'a str,
}

/// An item's weight: how many signals it holds, from which sources, over
/// what span of event times (34).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weight {
    pub signals: usize,
    /// Its signals' captures' sources, each once, in order.
    pub sources: Vec<String>,
    /// The earliest and latest of its signals' captures' event times, as
    /// written; none for an item with no signals.
    pub earliest: Option<String>,
    pub latest: Option<String>,
}

/// One item the inbox holds, with its brief and its sends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemAt {
    pub item: Item,
    /// The brief's text, after the line naming its format; none until one
    /// is written.
    pub brief: Option<String>,
    /// `sends.rec`'s records, in order.
    pub sends: Vec<SendRecord>,
}

impl ItemAt {
    /// Whether it is sent: proposed until its first send (36).
    pub fn is_sent(&self) -> bool {
        self.sends.iter().any(SendRecord::is_send)
    }
}

/// Where a move is checked: as a record read from the tip, or as one an
/// operation forms. A join formed for a sent item is refused (30), while a
/// join read from the tip may name an item sent since it was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Checked {
    Read,
    Formed,
}

/// What a move that is taken does beside standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    Stands,
    /// A join naming a name no item has had makes that item (34).
    MakesItem,
}

/// Why a move is not taken, with the field where there is one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotMoved {
    pub field: Option<&'static str>,
    pub why: String,
}

impl Held {
    /// Whether its signals are its own signal, written with it by `take`
    /// (19), rather than a reading: a capture taken with its own signal is
    /// never unread, while a reading leaves `unread` as the capture was
    /// written (41), and an import's signals are a reading (37).
    pub(crate) fn has_own_signal(&self) -> bool {
        self.signals.is_some()
            && !self.capture.unread
            && !matches!(self.capture.came_in, CameIn::Import(_))
    }
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
            moves: Vec::new(),
            by_signal: BTreeMap::new(),
            items: BTreeMap::new(),
            batches: BTreeMap::new(),
        };
        inbox.read_places(&places)?;
        Ok(inbox)
    }

    /// Reads each place in its order: every capture before every signals'
    /// place, every item before every brief and sends, so what lies beside
    /// a record is read before it. Moves are read last, against all of it.
    fn read_places(&mut self, places: &BTreeMap<Place, String>) -> Result<(), InboxError> {
        let mut moves = None;
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
                Place::Signals(key) => {
                    let Some(held) = self.captures.get_mut(key) else {
                        return Err(InboxError::at(
                            path,
                            "lies beside no `capture.rec`: a capture's signals lie beside the capture",
                        ));
                    };
                    held.signals = Some(read_signals(path, text, key)?);
                }
                Place::Moves => moves = Some(path),
                Place::Batch(name) => {
                    let batch = self.read_batch(path, text, name)?;
                    self.batches.insert(name.clone(), batch);
                }
                Place::Item(name) => {
                    let item = read_item(path, text, name)?;
                    self.items.insert(
                        name.clone(),
                        ItemAt {
                            item,
                            brief: None,
                            sends: Vec::new(),
                        },
                    );
                }
                Place::Brief(name) => {
                    let item = beside_item(&mut self.items, path, name)?;
                    let brief =
                        Format::read_brief(text).map_err(|reason| InboxError::at(path, reason))?;
                    item.brief = Some(brief.to_string());
                }
                Place::Sends(name) => {
                    let item = beside_item(&mut self.items, path, name)?;
                    item.sends = read_sends(path, text, name)?;
                }
            }
        }
        if let Some(path) = moves {
            self.read_moves(path)?;
        }
        // A batch that stands has every move of it in `moves.rec` (32).
        for (name, batch) in &self.batches {
            if batch.stands() != Some(true) {
                continue;
            }
            if let Some((n, missing)) = (2..)
                .zip(&batch.moves)
                .find(|(_, taken)| !self.moves.contains(taken))
            {
                return Err(InboxError::at(
                    Place::Batch(name.clone()).path(),
                    format!(
                        "stands, and its record {n}, a {} of `{}`, is not in `moves.rec`",
                        missing.word, missing.signal
                    ),
                ));
            }
        }
        Ok(())
    }

    /// The batch file at `path`, of the batch `name`: its proposal, then
    /// its moves, each naming the batch and a signal the inbox holds, no
    /// two for one signal, then at most one decision (32).
    fn read_batch(&self, path: &str, text: &str, name: &str) -> Result<Batch, InboxError> {
        let records = records(path, text, &[Format::Batch, Format::Move])?;
        let field = |n: usize, error| InboxError::field(path, Some(n), error);
        let names_it = |n: usize, batch: &str| {
            if batch == name {
                Ok(())
            } else {
                Err(field(
                    n,
                    RecError::field(
                        "batch",
                        format!("is `{batch}`, and the file is the batch `{name}`'s"),
                    ),
                ))
            }
        };
        let is_batch = |record: &Record| record.get("format") == Some(Format::Batch.as_str());
        let Some((first, rest)) = records.split_first().filter(|(first, _)| is_batch(first)) else {
            return Err(InboxError::at(
                path,
                "opens with no proposal: a batch's file opens with its `weir-batch/1` proposal",
            ));
        };
        let proposal = Proposal::from_record(first).map_err(|error| field(1, error))?;
        names_it(1, &proposal.batch)?;
        let mut moves: Vec<Move> = Vec::new();
        let mut decision = None;
        for (n, record) in (2..).zip(rest) {
            if decision.is_some() {
                return Err(InboxError::at(
                    path,
                    format!("holds record {n} after its decision, which is its last"),
                ));
            }
            if is_batch(record) {
                if record.get("stands").is_none() {
                    return Err(InboxError::at(
                        path,
                        format!("holds a second proposal, record {n}"),
                    ));
                }
                let decided = Decision::from_record(record).map_err(|error| field(n, error))?;
                names_it(n, &decided.batch)?;
                decision = Some(decided);
                continue;
            }
            let taken = Move::from_record(record).map_err(|error| field(n, error))?;
            names_it(n, taken.batch.as_deref().unwrap_or_default())?;
            if self.signal(&taken.signal).is_none() {
                return Err(field(
                    n,
                    RecError::field(
                        "signal",
                        format!("names `{}`, a signal the inbox does not hold", taken.signal),
                    ),
                ));
            }
            if moves.iter().any(|earlier| earlier.signal == taken.signal) {
                return Err(field(
                    n,
                    RecError::field(
                        "signal",
                        format!("names `{}`, which the batch moves already", taken.signal),
                    ),
                ));
            }
            moves.push(taken);
        }
        if moves.is_empty() {
            return Err(InboxError::at(
                path,
                "holds no move, and a batch is the moves proposed together",
            ));
        }
        Ok(Batch {
            proposal,
            moves,
            decision,
        })
    }

    /// Reads `moves.rec`, each move checked against those before it.
    fn read_moves(&mut self, path: &str) -> Result<(), InboxError> {
        let records = records(path, &self.files[path], &[Format::Move])?;
        for (n, record) in (1..).zip(&records) {
            let at = |field: Option<&str>, why: String| InboxError {
                path: path.to_string(),
                record: Some(n),
                field: field.map(str::to_string),
                reason: why,
            };
            let taken = Move::from_record(record)
                .map_err(|error| InboxError::field(path, Some(n), error))?;
            if let Some(name) = &taken.batch {
                let stood_in = self
                    .batches
                    .get(name)
                    .filter(|batch| batch.stands() == Some(true) && batch.moves.contains(&taken));
                if stood_in.is_none() {
                    return Err(at(
                        Some("batch"),
                        format!("is `{name}`, and no batch of that name stands holding this move"),
                    ));
                }
            }
            self.check_move(&taken, Checked::Read)
                .map_err(|not| at(not.field, not.why))?;
            self.push_move(taken);
        }
        Ok(())
    }

    /// Whether `taken` may stand on this inbox, and what it does beside:
    /// its signal held; a signal's first move naming no `replaces`, and
    /// every later one naming the standing move's number (29); a join's
    /// item held or, where formed, made, and never sent when formed; an
    /// attach's item sent (30).
    pub(crate) fn check_move(&self, taken: &Move, checked: Checked) -> Result<Effect, NotMoved> {
        let signal = &taken.signal;
        if self.signal(signal).is_none() {
            return Err(NotMoved {
                field: Some("signal"),
                why: format!("names `{signal}`, a signal the inbox does not hold"),
            });
        }
        match (self.standing(signal), taken.replaces) {
            (None, None) => {}
            (None, Some(replaces)) => {
                return Err(NotMoved {
                    field: Some("replaces"),
                    why: format!("names move {replaces}, and `{signal}` has no move to replace"),
                })
            }
            (Some((number, standing)), None) => {
                return Err(NotMoved {
                    field: None,
                    why: format!(
                    "`{signal}` has a standing move, {}; only a replacement naming it changes it",
                    standing.describe(number)
                ),
                })
            }
            (Some((number, standing)), Some(replaces)) if replaces != number => {
                return Err(NotMoved {
                    field: Some("replaces"),
                    why: format!(
                        "names move {replaces}, and the move standing is {}",
                        standing.describe(number)
                    ),
                })
            }
            (Some(_), Some(_)) => {}
        }
        let name = &taken.target;
        match (taken.word, self.items.get(name)) {
            (Word::Join, None) if checked == Checked::Formed => Ok(Effect::MakesItem),
            (Word::Join, None) => Err(NotMoved {
                field: Some("target"),
                why: format!("names `{name}`, which no item of the inbox is"),
            }),
            (Word::Join, Some(item)) if checked == Checked::Formed && item.is_sent() => {
                Err(NotMoved {
                    field: Some("target"),
                    why: format!(
                        "names `{name}`, which is sent: an attach is the move that adds to a sent item"
                    ),
                })
            }
            (Word::Attach, None) => Err(NotMoved {
                field: Some("target"),
                why: format!(
                    "names `{name}`, which no item of the inbox is, and an attach adds to a sent item"
                ),
            }),
            (Word::Attach, Some(item)) if !item.is_sent() => Err(NotMoved {
                field: Some("target"),
                why: format!(
                    "names `{name}`, which is not sent: a join is the move that grows a proposed item"
                ),
            }),
            _ => Ok(Effect::Stands),
        }
    }

    /// Adds `taken` to the moves, standing for its signal.
    pub(crate) fn push_move(&mut self, taken: Move) {
        self.by_signal
            .entry(taken.signal.clone())
            .or_default()
            .push(self.moves.len());
        self.moves.push(taken);
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

    /// Whether the capture under `key` is unread: held, with no own
    /// signal and no reading (19, 28).
    pub fn is_unread(&self, key: &Key) -> bool {
        self.captures
            .get(key)
            .is_some_and(|held| held.signals.is_none())
    }

    /// The standing move of the signal `id`, its last, with its number
    /// among the signal's moves, from 1 (29).
    pub fn standing(&self, id: &SignalId) -> Option<(u64, &Move)> {
        let places = self.by_signal.get(id)?;
        let last = *places.last()?;
        Some((places.len() as u64, &self.moves[last]))
    }

    /// Every move of `moves.rec`, in order.
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Every signal with no standing move, in key and reading order, each
    /// with its capture's event time, which its age is read from, however
    /// old (33; core.8). A move in a proposed batch does not stand (32).
    pub fn unmoved(&self) -> Vec<Unmoved<'_>> {
        self.captures
            .values()
            .flat_map(|held| {
                held.signals.iter().flatten().map(|signal| Unmoved {
                    signal,
                    event_at: &held.capture.event_at,
                })
            })
            .filter(|unmoved| !self.by_signal.contains_key(&unmoved.signal.signal))
            .collect()
    }

    /// The signals of the item `name`: exactly those whose standing move is
    /// a join or an attach naming it, read from the moves whenever asked
    /// and recorded nowhere (34; core.20), in signal order.
    pub fn item_signals(&self, name: &str) -> Vec<&SignalId> {
        self.by_signal
            .iter()
            .filter(|(_, places)| {
                places.last().is_some_and(|last| {
                    let standing = &self.moves[*last];
                    standing.word.names_an_item() && standing.target == name
                })
            })
            .map(|(id, _)| id)
            .collect()
    }

    /// The weight of the item `name`, computed from its signals and never
    /// stored (34; core.20): how many, their captures' sources, and the
    /// earliest and latest of their captures' event times, compared as
    /// instants and given as written (core.8). None for an item not held.
    pub fn weight(&self, name: &str) -> Option<Weight> {
        self.items.get(name)?;
        let signals = self.item_signals(name);
        let mut weight = Weight {
            signals: signals.len(),
            sources: Vec::new(),
            earliest: None,
            latest: None,
        };
        let mut span: Option<((OffsetDateTime, &str), (OffsetDateTime, &str))> = None;
        for id in signals {
            let Some(capture) = self.capture(&id.key) else {
                continue;
            };
            if !weight.sources.contains(&capture.source) {
                weight.sources.push(capture.source.clone());
            }
            let Ok(instant) = OffsetDateTime::parse(&capture.event_at, &Rfc3339) else {
                continue;
            };
            let at = (instant, capture.event_at.as_str());
            span = Some(match span {
                None => (at, at),
                Some((earliest, latest)) => (
                    if instant < earliest.0 { at } else { earliest },
                    if instant > latest.0 { at } else { latest },
                ),
            });
        }
        weight.sources.sort();
        if let Some(((_, earliest), (_, latest))) = span {
            weight.earliest = Some(earliest.to_string());
            weight.latest = Some(latest.to_string());
        }
        Some(weight)
    }

    /// The batch named `name`, as its file holds it.
    pub fn batch(&self, name: &str) -> Option<&Batch> {
        self.batches.get(name)
    }

    /// The item named `name`, with its brief and sends.
    pub fn item(&self, name: &str) -> Option<&ItemAt> {
        self.items.get(name)
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

/// The one `weir-item/1` record of the `item.rec` at `path`, which names
/// the item its folder is.
fn read_item(path: &str, text: &str, name: &str) -> Result<Item, InboxError> {
    let records = records(path, text, &[Format::Item])?;
    let [record] = records.as_slice() else {
        return Err(InboxError::at(
            path,
            format!(
                "holds {} records, and an item's record is one",
                records.len()
            ),
        ));
    };
    let item =
        Item::from_record(record).map_err(|error| InboxError::field(path, Some(1), error))?;
    if item.item != name {
        return Err(InboxError::field(
            path,
            Some(1),
            RecError::field(
                "item",
                format!(
                    "is `{}`, and the item lies in the folder of `{name}`",
                    item.item
                ),
            ),
        ));
    }
    Ok(item)
}

/// The item a brief or a sends file at `path` lies beside.
fn beside_item<'a>(
    items: &'a mut BTreeMap<String, ItemAt>,
    path: &str,
    name: &str,
) -> Result<&'a mut ItemAt, InboxError> {
    items.get_mut(name).ok_or_else(|| {
        InboxError::at(
            path,
            "lies beside no `item.rec`: an item's brief and sends lie beside the item",
        )
    })
}

/// The records of the `sends.rec` at `path`, of the item `name`: each send
/// once to a target at a time, and what became of each after it, once.
fn read_sends(path: &str, text: &str, name: &str) -> Result<Vec<SendRecord>, InboxError> {
    let records = records(path, text, &[Format::Send])?;
    let mut sends: Vec<SendRecord> = Vec::with_capacity(records.len());
    for (n, record) in (1..).zip(&records) {
        let at = |field: &str, reason: String| {
            InboxError::field(path, Some(n), RecError::field(field, reason))
        };
        let read = SendRecord::from_record(record)
            .map_err(|error| InboxError::field(path, Some(n), error))?;
        if read.item != name {
            return Err(at(
                "item",
                format!(
                    "is `{}`, and the sends lie in the folder of `{name}`",
                    read.item
                ),
            ));
        }
        let names_it = |earlier: &&SendRecord| {
            earlier.target == read.target && earlier.sent_at == read.sent_at
        };
        let send = sends.iter().filter(|r| r.is_send()).find(names_it);
        let settled = sends.iter().filter(|r| !r.is_send()).find(names_it);
        match (read.is_send(), send, settled) {
            (true, Some(_), _) => {
                return Err(at(
                    "sent_at",
                    format!(
                        "is `{}`, and the item was sent to `{}` then already",
                        read.sent_at, read.target
                    ),
                ))
            }
            (false, None, _) => {
                return Err(at(
                    "sent_at",
                    format!(
                        "is `{}`, and no send to `{}` then comes before it",
                        read.sent_at, read.target
                    ),
                ))
            }
            (false, Some(_), Some(earlier)) => {
                return Err(at(
                    "outcome",
                    format!(
                        "settles the send to `{}` at `{}`, settled already as `{}`",
                        read.target, read.sent_at, earlier.outcome
                    ),
                ))
            }
            _ => {}
        }
        sends.push(read);
    }
    Ok(sends)
}
