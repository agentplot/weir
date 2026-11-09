//! A delivery's records in recutils, as on a shared branch (C.8), written
//! as C.2 writes records through `weir_capture::rec`: in the order given,
//! `format` first and then each field in C.8's order, one `reader` field
//! per reader of a delivery, a message's later lines as `+` lines. What it
//! writes passes `recfix` (core.24), which the crate's tests hold it to.

use weir_capture::rec;

use crate::calls::Refusal;
use crate::record::{
    Confirmation, Delivery, Handover, Known, Record, CONFIRMATION, DELIVERY, HANDOVER, KNOWN,
};
use crate::text::{Key, Message, Reader, Time, Way};

/// The fields each format names, in C.8's order, `format` first.
pub(crate) fn fields_of(format: &str) -> Option<&'static [&'static str]> {
    match format {
        DELIVERY => Some(&[
            "format",
            "key",
            "reader",
            "message",
            "recorded_at",
            "expires_at",
        ]),
        HANDOVER => Some(&["format", "key", "reader", "by", "at"]),
        CONFIRMATION => Some(&["format", "key", "reader", "at"]),
        KNOWN => Some(&["format", "reader", "known_at"]),
        _ => None,
    }
}

/// The records as recutils text, in the order given (C.8).
pub fn write(records: &[Record]) -> String {
    let written: Vec<rec::Record> = records.iter().map(spelled).collect();
    rec::write(&written)
}

fn spelled(record: &Record) -> rec::Record {
    let mut spelled = rec::Record::new();
    spelled.push("format", record.format());
    match record {
        Record::Delivery(delivery) => {
            spelled.push("key", delivery.key().as_str());
            for reader in delivery.readers() {
                spelled.push("reader", reader.as_str());
            }
            spelled.push("message", delivery.message().as_str());
            spelled.push("recorded_at", delivery.recorded_at().as_str());
            spelled.push("expires_at", delivery.expires_at().as_str());
        }
        Record::Handover(handover) => {
            spelled.push("key", handover.key.as_str());
            spelled.push("reader", handover.reader.as_str());
            spelled.push("by", handover.by.as_str());
            spelled.push("at", handover.at.as_str());
        }
        Record::Confirmation(confirmation) => {
            spelled.push("key", confirmation.key.as_str());
            spelled.push("reader", confirmation.reader.as_str());
            spelled.push("at", confirmation.at.as_str());
        }
        Record::Known(known) => {
            spelled.push("reader", known.reader.as_str());
            spelled.push("known_at", known.known_at.as_str());
        }
    }
    spelled
}

/// The records of recutils text, in order; or a refusal naming the field:
/// a format the crate does not read, a field missing, a field the format
/// does not name, or one given twice where C.8 gives it once (40; C.8;
/// core.22).
pub fn read(text: &str) -> Result<Vec<Record>, Refusal> {
    let read = rec::read(text).map_err(|error| match error.field {
        Some(field) => Refusal::new(field, error.reason),
        None => Refusal::new("text", error.reason),
    })?;
    read.iter()
        .enumerate()
        .map(|(n, record)| Fields { record, n: n + 1 }.record())
        .collect()
}

/// One record of the text, and its place among them, so a refusal says
/// which.
struct Fields<'a> {
    record: &'a rec::Record,
    n: usize,
}

impl Fields<'_> {
    fn refused(&self, field: &str, reason: impl std::fmt::Display) -> Refusal {
        Refusal::new(field, format!("{reason}, in record {}", self.n))
    }

    fn one(&self, field: &str) -> Result<&str, Refusal> {
        self.record
            .required(field)
            .map_err(|error| self.refused(field, error.reason))
    }

    fn time(&self, field: &str) -> Result<Time, Refusal> {
        Time::parse(self.one(field)?, field).map_err(|refused| self.refused(field, refused.reason))
    }

    fn key(&self) -> Result<Key, Refusal> {
        Key::parse(self.one("key")?).map_err(|refused| self.refused("key", refused.reason))
    }

    fn reader(&self) -> Result<Reader, Refusal> {
        Reader::parse(self.one("reader")?).map_err(|refused| self.refused("reader", refused.reason))
    }

    fn record(&self) -> Result<Record, Refusal> {
        let format = self.one("format")?;
        let fields = fields_of(format).ok_or_else(|| {
            self.refused("format", format!("`{format}` is no format the crate reads"))
        })?;
        for (name, _) in self.record.fields() {
            if !fields.contains(&name.as_str()) {
                return Err(self.refused(name, format!("is no field of `{format}`")));
            }
        }
        Ok(match format {
            DELIVERY => {
                let key = self.key()?;
                let readers = self.record.get_all("reader");
                if readers.is_empty() {
                    return Err(self.refused("reader", "is missing"));
                }
                let readers = readers
                    .into_iter()
                    .map(|text| {
                        Reader::parse(text)
                            .map_err(|refused| self.refused("reader", refused.reason))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let message = Message::parse(self.one("message")?)
                    .map_err(|refused| self.refused("message", refused.reason))?;
                Delivery::read(
                    key,
                    readers,
                    message,
                    self.time("recorded_at")?,
                    self.time("expires_at")?,
                    "reader",
                )
                .map_err(|refused| self.refused(&refused.field, refused.reason))?
                .into()
            }
            HANDOVER => Handover {
                key: self.key()?,
                reader: self.reader()?,
                by: Way::parse(self.one("by")?)
                    .map_err(|refused| self.refused("by", refused.reason))?,
                at: self.time("at")?,
            }
            .into(),
            CONFIRMATION => Confirmation {
                key: self.key()?,
                reader: self.reader()?,
                at: self.time("at")?,
            }
            .into(),
            _ => Known {
                reader: self.reader()?,
                known_at: self.time("known_at")?,
            }
            .into(),
        })
    }
}
