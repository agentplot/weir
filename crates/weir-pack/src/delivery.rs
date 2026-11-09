//! The delivery shape's cases (C.7, C.8): each holds, in its folder,
//! `case.toml`, naming its clauses, one call of B.7 with its arguments and
//! what must come of it; `start.rec`, the store's starting records in
//! recutils, which may be absent; and, where the call adds records,
//! `added.rec`, the records it adds in order. A field the format does not
//! name fails the read, naming the case and the field.
//!
//! A case names exactly one thing that must come of its call: the records
//! added (`added.rec`); that the key is held (`held = true`, for `record`);
//! when the reader first confirmed (`already`, for `hand` and `confirm`);
//! the refusal, by its field and a phrase of its reason (`[refused]`); or,
//! for `owed`, each reader owed with its deliveries and the keys done
//! (`done` and `[[owed]]`).
//!
//! The run holds a deliverer's store to the cases through its read and add
//! alone (112): a case runs against weir's crate, through a store its
//! caller hands it (65, 87, 113).

use serde::Deserialize;
use toml::Value;
use weir_delivery::{
    json, rec, Confirmed, Delivery, Handed, Owed, Record, Recorded, Refusal, Store, Time,
};

use crate::clauses::Clause;
use crate::pack::{Case, Pack, Shape};

/// The files a delivery case may hold.
const FILES: [&str; 3] = ["case.toml", "start.rec", "added.rec"];

/// A delivery case, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryCase {
    /// `delivery/<name>`.
    pub id: String,
    pub name: String,
    pub satisfies: Vec<Clause>,
    pub call: Call,
    /// The store's records before the call, in the order added.
    pub start: Vec<Record>,
    pub expected: Outcome,
}

/// The one call of B.7 a case makes, its arguments as the case writes them,
/// so the call parses them and a refusal names the field (108 to 111).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Record {
        key: String,
        message: String,
        readers: Vec<String>,
        at: String,
        span: u64,
    },
    Hand {
        key: String,
        reader: String,
        by: String,
        at: String,
        confirms: bool,
    },
    Confirm {
        key: String,
        reader: String,
        at: String,
    },
    Owed {
        now: String,
    },
}

impl Call {
    /// The call's name, as a case writes it under `call`.
    pub fn name(&self) -> &'static str {
        match self {
            Call::Record { .. } => "record",
            Call::Hand { .. } => "hand",
            Call::Confirm { .. } => "confirm",
            Call::Owed { .. } => "owed",
        }
    }
}

/// The one thing that must come of a case's call (C.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The records the call adds, in order, after the starting records.
    Added(Vec<Record>),
    /// The key is held, the delivery the starting records hold under it
    /// standing (101).
    Held,
    /// The reader has confirmed: when it first did (102, 103).
    Already(String),
    /// The refusal: its field, and a phrase its reason holds.
    Refused { field: String, reason_holds: String },
    /// What `owed` gives: each reader owed, in order, and the keys done.
    Owed {
        readers: Vec<ExpectedReader>,
        done: Vec<String>,
    },
}

/// One reader `owed` must give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedReader {
    pub reader: String,
    pub known: bool,
    pub deliveries: Vec<ExpectedDelivery>,
}

/// One delivery a reader is owed: its key, whether expired, and the ways of
/// its hand-overs to that reader, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedDelivery {
    pub key: String,
    pub expired: bool,
    pub handed: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    satisfies: Vec<Value>,
    call: WrittenCall,
    held: Option<bool>,
    already: Option<String>,
    refused: Option<WrittenRefused>,
    done: Option<Vec<String>>,
    owed: Option<Vec<WrittenReader>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenCall {
    record: Option<WrittenRecord>,
    hand: Option<WrittenHand>,
    confirm: Option<WrittenConfirm>,
    owed: Option<WrittenOwed>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRecord {
    key: String,
    message: String,
    readers: Vec<String>,
    at: String,
    span: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenHand {
    key: String,
    reader: String,
    by: String,
    at: String,
    confirms: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenConfirm {
    key: String,
    reader: String,
    at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenOwed {
    now: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRefused {
    field: String,
    reason_holds: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenReader {
    reader: String,
    known: bool,
    delivery: Vec<WrittenDelivery>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenDelivery {
    key: String,
    expired: bool,
    #[serde(default)]
    handed: Vec<String>,
}

/// Reads the delivery case `case`, or names the case and why it does not
/// read: a field it fails on, no call or two, a file it holds that a
/// delivery case does not, records that do not read, no expectation or
/// two, or one its call cannot give.
pub fn read_delivery_case(case: &Case) -> Result<DeliveryCase, String> {
    let id = case.id();
    let refused = |why: String| format!("{id}: {why}");
    if let Some(stray) = case
        .files
        .keys()
        .find(|path| !FILES.contains(&path.as_str()))
    {
        return Err(refused(format!(
            "holds `{stray}`, which is no file of a delivery case: {}",
            FILES.join(", ")
        )));
    }
    let written: Written = toml::from_str(case.text("case.toml")?)
        .map_err(|error| refused(error.message().to_string()))?;
    let satisfies = written
        .satisfies
        .iter()
        .map(Clause::cited)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|why| refused(format!("`satisfies`: {why}")))?;
    if satisfies.is_empty() {
        return Err(refused("cites no clause in `satisfies`".into()));
    }
    let WrittenCall {
        record,
        hand,
        confirm,
        owed,
    } = written.call;
    let mut calls = Vec::new();
    calls.extend(record.map(|call| Call::Record {
        key: call.key,
        message: call.message,
        readers: call.readers,
        at: call.at,
        span: call.span,
    }));
    calls.extend(hand.map(|call| Call::Hand {
        key: call.key,
        reader: call.reader,
        by: call.by,
        at: call.at,
        confirms: call.confirms,
    }));
    calls.extend(confirm.map(|call| Call::Confirm {
        key: call.key,
        reader: call.reader,
        at: call.at,
    }));
    calls.extend(owed.map(|call| Call::Owed { now: call.now }));
    let call = match <[Call; 1]>::try_from(calls) {
        Ok([call]) => call,
        Err(_) => {
            return Err(refused(
                "names no call or more than one: exactly one of `call.record`, `call.hand`, \
                 `call.confirm` and `call.owed`"
                    .into(),
            ))
        }
    };
    let records = |path: &str| -> Result<Option<Vec<Record>>, String> {
        match case.file(path) {
            None => Ok(None),
            Some(_) => rec::read(case.text(path)?)
                .map(Some)
                .map_err(|refusal| refused(format!("`{path}`: {refusal}"))),
        }
    };
    let start = records("start.rec")?.unwrap_or_default();
    let mut expected = Vec::new();
    if let Some(added) = records("added.rec")? {
        expected.push(Outcome::Added(added));
    }
    match written.held {
        Some(true) => expected.push(Outcome::Held),
        Some(false) => {
            return Err(refused(
                "`held` is false, and a case that expects the key held says `held = true`".into(),
            ))
        }
        None => {}
    }
    if let Some(time) = written.already {
        expected.push(Outcome::Already(time));
    }
    if let Some(refusal) = written.refused {
        expected.push(Outcome::Refused {
            field: refusal.field,
            reason_holds: refusal.reason_holds,
        });
    }
    if written.done.is_some() || written.owed.is_some() {
        expected.push(Outcome::Owed {
            readers: written
                .owed
                .unwrap_or_default()
                .into_iter()
                .map(|owed| ExpectedReader {
                    reader: owed.reader,
                    known: owed.known,
                    deliveries: owed
                        .delivery
                        .into_iter()
                        .map(|delivery| ExpectedDelivery {
                            key: delivery.key,
                            expired: delivery.expired,
                            handed: delivery.handed,
                        })
                        .collect(),
                })
                .collect(),
            done: written.done.unwrap_or_default(),
        });
    }
    let expected = match <[Outcome; 1]>::try_from(expected) {
        Ok([outcome]) => outcome,
        Err(_) => {
            return Err(refused(
                "names no expectation or more than one: exactly one of `added.rec`, `held`, \
                 `already`, `[refused]`, and `done` with `[[owed]]`"
                    .into(),
            ))
        }
    };
    let fits = match (&call, &expected) {
        (_, Outcome::Refused { .. }) => true,
        (Call::Owed { .. }, expected) => matches!(expected, Outcome::Owed { .. }),
        (Call::Record { .. }, expected) => matches!(expected, Outcome::Added(_) | Outcome::Held),
        (Call::Hand { .. } | Call::Confirm { .. }, expected) => {
            matches!(expected, Outcome::Added(_) | Outcome::Already(_))
        }
    };
    if !fits {
        return Err(refused(format!(
            "expects what `{}` never gives",
            call.name()
        )));
    }
    Ok(DeliveryCase {
        id: id.clone(),
        name: case.name.clone(),
        satisfies,
        call,
        start,
        expected,
    })
}

/// What a case's call answered.
enum Answer {
    Add(Vec<Record>),
    Held(Delivery),
    Already(Time),
    Refused(Refusal),
    Owed(Owed),
}

impl Answer {
    fn said(&self) -> String {
        match self {
            Answer::Add(records) => format!("{} records to add", records.len()),
            Answer::Held(_) => "the key held".into(),
            Answer::Already(at) => format!("that the reader confirmed at {at}"),
            Answer::Refused(refusal) => format!("a refusal, {refusal}"),
            Answer::Owed(_) => "what is owed".into(),
        }
    }
}

impl Outcome {
    fn said(&self) -> String {
        match self {
            Outcome::Added(records) => format!("{} records added", records.len()),
            Outcome::Held => "the key held".into(),
            Outcome::Already(at) => format!("that the reader confirmed at {at}"),
            Outcome::Refused {
                field,
                reason_holds,
            } => format!("`{field}` refused for a reason holding \"{reason_holds}\""),
            Outcome::Owed { .. } => "what is owed".into(),
        }
    }
}

/// The call `case` makes, with weir's crate, over `records`.
fn answer(call: &Call, records: &[Record]) -> Answer {
    let answered = match call {
        Call::Record {
            key,
            message,
            readers,
            at,
            span,
        } => {
            let readers: Vec<&str> = readers.iter().map(String::as_str).collect();
            weir_delivery::record(records, key, message, &readers, at, *span).map(|recorded| {
                match recorded {
                    Recorded::Add(add) => Answer::Add(add),
                    Recorded::Held(delivery) => Answer::Held(delivery),
                }
            })
        }
        Call::Hand {
            key,
            reader,
            by,
            at,
            confirms,
        } => weir_delivery::hand(records, key, reader, by, at, *confirms).map(
            |handed| match handed {
                Handed::Add(add) => Answer::Add(add),
                Handed::Confirmed(at) => Answer::Already(at),
            },
        ),
        Call::Confirm { key, reader, at } => {
            weir_delivery::confirm(records, key, reader, at).map(|confirmed| match confirmed {
                Confirmed::Add(add) => Answer::Add(add),
                Confirmed::Already(at) => Answer::Already(at),
            })
        }
        Call::Owed { now } => weir_delivery::owed(records, now).map(Answer::Owed),
    };
    answered.unwrap_or_else(Answer::Refused)
}

/// Each record of `expected` a store did not read back as written, and
/// each it read back that is not expected, by its JSON line (C.8).
fn differences(expected: &[Record], read: &[Record]) -> Vec<String> {
    let mut unmatched: Vec<Option<&Record>> = read.iter().map(Some).collect();
    let mut said = Vec::new();
    for record in expected {
        match unmatched.iter().position(|left| *left == Some(record)) {
            Some(at) => unmatched[at] = None,
            None => said.push(format!("not read back: {}", json::write(record))),
        }
    }
    said.extend(
        unmatched
            .into_iter()
            .flatten()
            .map(|record| format!("not expected: {}", json::write(record))),
    );
    if said.is_empty() && expected != read {
        said.push("read back in another order than added".into());
    }
    said
}

/// What `owed` gave, in the case's terms.
fn as_expected(owed: &Owed) -> (Vec<ExpectedReader>, Vec<String>) {
    let readers = owed
        .readers
        .iter()
        .map(|owed| ExpectedReader {
            reader: owed.reader.to_string(),
            known: owed.known,
            deliveries: owed
                .deliveries
                .iter()
                .map(|owed| ExpectedDelivery {
                    key: owed.delivery.key().to_string(),
                    expired: owed.expired,
                    handed: owed
                        .handovers
                        .iter()
                        .map(|handover| handover.by.to_string())
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let done = owed.done.iter().map(ToString::to_string).collect();
    (readers, done)
}

/// Readers owed, in words: each with whether its name is known and each
/// delivery's key, owed or expired, and the ways it was handed over.
fn said_of(readers: &[ExpectedReader]) -> String {
    if readers.is_empty() {
        return "no reader".into();
    }
    readers
        .iter()
        .map(|reader| {
            let deliveries: Vec<String> = reader
                .deliveries
                .iter()
                .map(|delivery| {
                    let mut said = format!(
                        "`{}` {}",
                        delivery.key,
                        if delivery.expired { "expired" } else { "owed" }
                    );
                    if !delivery.handed.is_empty() {
                        said.push_str(&format!(", handed by {}", delivery.handed.join(", ")));
                    }
                    said
                })
                .collect();
            format!(
                "`{}`, {}: {}",
                reader.reader,
                if reader.known { "known" } else { "not known" },
                deliveries.join("; ")
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Runs `case` through `store`, an empty store its caller hands it, as C.7
/// says: adds the starting records and reads them back, makes the case's
/// call with weir's crate over what it read, adds what the call gives, and
/// reads the store back (65, 87, 112, 113). Compares the store with the
/// starting records followed by those the case expects added, record by
/// record, and the answer with the case's; or names every difference, a
/// record by its JSON line (S25; core.52).
pub fn run_delivery_case<S: Store>(case: &DeliveryCase, store: &mut S) -> Result<(), Vec<String>> {
    let mut said: Vec<String> = Vec::new();
    let mut say = |difference: String| {
        if !said.contains(&difference) {
            said.push(difference);
        }
    };
    if let Err(error) = store.add(&case.start) {
        return Err(vec![format!(
            "the store's add of the starting records failed: {error}"
        )]);
    }
    let read = match store.read() {
        Ok(read) => read,
        Err(error) => return Err(vec![format!("the store's read failed: {error}")]),
    };
    differences(&case.start, &read)
        .into_iter()
        .for_each(&mut say);
    let answer = answer(&case.call, &read);
    if let Answer::Add(add) = &answer {
        if let Err(error) = store.add(add) {
            say(format!(
                "the store's add of what `{}` gave failed: {error}",
                case.call.name()
            ));
            return Err(said);
        }
    }
    let mut expected = case.start.clone();
    if let Outcome::Added(added) = &case.expected {
        expected.extend(added.iter().cloned());
    }
    match store.read() {
        Ok(read) => differences(&expected, &read).into_iter().for_each(&mut say),
        Err(error) => say(format!("the store's read failed: {error}")),
    }
    let call = case.call.name();
    match (&case.expected, &answer) {
        (Outcome::Added(_), Answer::Add(_)) => {}
        (Outcome::Held, Answer::Held(delivery)) => {
            let Call::Record { key, .. } = &case.call else {
                unreachable!("a case expecting the key held makes `record`")
            };
            let standing = case.start.iter().find_map(|record| match record {
                Record::Delivery(standing) if standing.key().as_str() == key => Some(standing),
                _ => None,
            });
            if standing != Some(delivery) {
                say(format!(
                    "`record` held the key with {}, and the starting records hold {}",
                    json::write(&Record::Delivery(delivery.clone())),
                    standing.map_or("no delivery under it".into(), |standing| json::write(
                        &Record::Delivery(standing.clone())
                    ))
                ));
            }
        }
        (Outcome::Already(expected), Answer::Already(at)) => {
            if at.as_str() != expected {
                say(format!(
                    "`{call}` said the reader confirmed at {at}, and the case expects {expected}"
                ));
            }
        }
        (
            Outcome::Refused {
                field,
                reason_holds,
            },
            Answer::Refused(refusal),
        ) => {
            if refusal.field != *field || !refusal.reason.contains(reason_holds.as_str()) {
                say(format!(
                    "`{call}` gave a refusal, {refusal}, and the case expects {}",
                    case.expected.said()
                ));
            }
        }
        (Outcome::Owed { readers, done }, Answer::Owed(owed)) => {
            let (gave, gave_done) = as_expected(owed);
            if gave != *readers {
                say(format!(
                    "`owed` gave {}, and the case expects {}",
                    said_of(&gave),
                    said_of(readers)
                ));
            }
            if gave_done != *done {
                say(format!(
                    "`owed` gave done [{}], and the case expects [{}]",
                    gave_done.join(", "),
                    done.join(", ")
                ));
            }
        }
        (expected, answer) => say(format!(
            "`{call}` gave {}, and the case expects {}",
            answer.said(),
            expected.said()
        )),
    }
    if said.is_empty() {
        Ok(())
    } else {
        Err(said)
    }
}

/// Runs every delivery case of `pack`, each through a fresh store from
/// `make`, and gives each case's name, `delivery/<name>`, with its
/// verdict; a case that does not read fails, saying why (87, 113). A
/// deliverer calls it in its own tests over its own store, with
/// `Pack::embedded()`.
pub fn conform_deliveries<S: Store>(
    pack: &Pack,
    mut make: impl FnMut() -> S,
) -> Vec<(String, Result<(), Vec<String>>)> {
    pack.cases(Shape::Delivery)
        .iter()
        .map(|case| match read_delivery_case(case) {
            Err(why) => (case.id(), Err(vec![why])),
            Ok(read) => {
                let verdict = run_delivery_case(&read, &mut make());
                (read.id, verdict)
            }
        })
        .collect()
}
