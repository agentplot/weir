//! The calls a deliverer makes over its own records (B.7), and the refusal
//! each gives naming the field that failed.

use std::fmt;

use crate::record::{self, Confirmation, Delivery, Known, Record};
use crate::text::{Key, Message, Reader, Span, Time};

/// A call, or a read of a record, refused: the field that failed and why
/// (100, 108 to 110; core.47).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// The field refused, as the call's argument or the record's field is
    /// named: `key`, `readers`, `at`, `span` and the like.
    pub field: String,
    /// Why, in words a reader of the refusal is shown.
    pub reason: String,
}

impl Refusal {
    /// The refusal of `field`, for `reason`.
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Refusal {
        Refusal {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`: {}", self.field, self.reason)
    }
}

impl std::error::Error for Refusal {}

/// What `record` gives (108): the delivery's record to add, or that the
/// key is held, with the delivery that stands (101).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recorded {
    /// The one `weir-delivery/1` record to add.
    Add(Vec<Record>),
    /// The key is held: the first delivery the records hold under it,
    /// whatever this call carried. Nothing is to be added.
    Held(Delivery),
}

/// The first delivery `records` hold under `key`, which is the one that
/// stands (101).
pub(crate) fn delivery_under<'a>(records: &'a [Record], key: &Key) -> Option<&'a Delivery> {
    records.iter().find_map(|record| match record {
        Record::Delivery(delivery) if delivery.key() == key => Some(delivery),
        _ => None,
    })
}

/// `record` (108): the delivery of `message` to `readers` under `key`,
/// recorded `at` and expiring `span` seconds after it (100, 104); or, where
/// `records` hold a delivery under `key`, that delivery, adding nothing
/// (101). A call is refused at the first field that fails, in the order of
/// its arguments: a key, a message or a reader out of its shape, no reader
/// or one named twice (`readers`), a time out of its shape (`at`), and a
/// span of 0 seconds (`span`) (core.47).
pub fn record(
    records: &[Record],
    key: &str,
    message: &str,
    readers: &[&str],
    at: &str,
    span: u64,
) -> Result<Recorded, Refusal> {
    let key = Key::parse(key)?;
    let message = Message::parse(message)?;
    let readers = readers
        .iter()
        .map(|reader| Reader::parse_as(reader, "readers"))
        .collect::<Result<Vec<_>, _>>()?;
    let readers = record::readers(readers, "readers")?;
    let at = Time::parse(at, "at")?;
    let span = Span::seconds(span)?;
    let delivery = Delivery::new(key, message, readers, at, span)?;
    Ok(match delivery_under(records, delivery.key()) {
        Some(held) => Recorded::Held(held.clone()),
        None => Recorded::Add(vec![delivery.into()]),
    })
}

/// What `confirm` gives (110): the records to add, or that the reader has
/// confirmed the delivery, and when it first did (102).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirmed {
    /// The `weir-confirmation/1` record and, where no `weir-known/1` record
    /// names the reader, one making its name known, in that order.
    Add(Vec<Record>),
    /// The reader has confirmed the delivery: when it first did. Nothing is
    /// to be added.
    Already(Time),
}

/// The delivery `records` hold under `key`, where `reader` is one of its
/// readers; or a refusal naming `key` when no delivery is under it, or
/// `reader` when the delivery does not name it (102, 103).
fn delivery_to<'a>(
    records: &'a [Record],
    key: &Key,
    reader: &Reader,
) -> Result<&'a Delivery, Refusal> {
    let delivery = delivery_under(records, key)
        .ok_or_else(|| Refusal::new("key", format!("no delivery is recorded under `{key}`")))?;
    if !delivery.readers().contains(reader) {
        return Err(Refusal::new(
            "reader",
            format!("`{reader}` is not a reader of `{key}`"),
        ));
    }
    Ok(delivery)
}

/// When `reader` first confirmed the delivery under `key`, where the
/// records hold its confirmation.
fn first_confirmed<'a>(records: &'a [Record], key: &Key, reader: &Reader) -> Option<&'a Time> {
    records.iter().find_map(|record| match record {
        Record::Confirmation(confirmation)
            if confirmation.key == *key && confirmation.reader == *reader =>
        {
            Some(&confirmation.at)
        }
        _ => None,
    })
}

/// The records `reader`'s confirmation of `key` at `at` adds: the
/// confirmation, and its name made known where no `weir-known/1` record of
/// `records` names it (106; core.50).
fn confirming(records: &[Record], key: Key, reader: Reader, at: Time) -> Vec<Record> {
    let known = records
        .iter()
        .any(|record| matches!(record, Record::Known(known) if known.reader == reader));
    let mut add = vec![Confirmation {
        key,
        reader: reader.clone(),
        at: at.clone(),
    }
    .into()];
    if !known {
        add.push(
            Known {
                reader,
                known_at: at,
            }
            .into(),
        );
    }
    add
}

/// `confirm` (110): `reader`'s confirmation of the delivery under `key`
/// `at`, with its name made known where it was not (102, 106); or, where
/// the records hold its confirmation of `key`, when it first confirmed,
/// adding nothing. An expired delivery is confirmed as an owed one (104).
/// A call is refused at the first field out of its shape, `key`, `reader`
/// or `at`, then naming `key` where no delivery is under it, or `reader`
/// where the delivery does not name it (core.48).
pub fn confirm(
    records: &[Record],
    key: &str,
    reader: &str,
    at: &str,
) -> Result<Confirmed, Refusal> {
    let key = Key::parse(key)?;
    let reader = Reader::parse(reader)?;
    let at = Time::parse(at, "at")?;
    delivery_to(records, &key, &reader)?;
    if let Some(first) = first_confirmed(records, &key, &reader) {
        return Ok(Confirmed::Already(first.clone()));
    }
    Ok(Confirmed::Add(confirming(records, key, reader, at)))
}
