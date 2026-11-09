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

use serde::Deserialize;
use toml::Value;
use weir_delivery::{rec, Record};

use crate::clauses::Clause;
use crate::pack::Case;

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
