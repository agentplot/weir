//! A delivery's records in JSON, in flight and in a store on a machine
//! (C.8): each one object on one line, `format` first and its fields as C.8
//! names them, a delivery's readers a list, `readers`, spelled as C.8's
//! example spells them, `": "` after a name and `", "` between fields.

use std::io;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::calls::Refusal;
use crate::record::{
    Confirmation, Delivery, Handover, Known, Record, CONFIRMATION, DELIVERY, HANDOVER, KNOWN,
};
use crate::text::{Key, Message, Reader, Time, Way};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeliveryLine {
    format: String,
    key: String,
    readers: Vec<String>,
    message: String,
    recorded_at: String,
    expires_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HandoverLine {
    format: String,
    key: String,
    reader: String,
    by: String,
    at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmationLine {
    format: String,
    key: String,
    reader: String,
    at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnownLine {
    format: String,
    reader: String,
    known_at: String,
}

/// The fields each format names in JSON, in C.8's order, `format` first.
fn fields_of(format: &str) -> Option<&'static [&'static str]> {
    match format {
        DELIVERY => Some(&[
            "format",
            "key",
            "readers",
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

/// C.8's spelling: a space after each `:` and `,` of an object or a list.
struct Spaced;

impl serde_json::ser::Formatter for Spaced {
    fn begin_object_key<W: ?Sized + io::Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_value<W: ?Sized + io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b": ")
    }

    fn begin_array_value<W: ?Sized + io::Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }
}

fn spelled(line: &impl Serialize) -> String {
    let mut bytes = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, Spaced);
    line.serialize(&mut serializer)
        .expect("a record of strings is always JSON");
    String::from_utf8(bytes).expect("JSON is UTF-8")
}

/// The record as one line of JSON, with no line end (C.8).
pub fn write(record: &Record) -> String {
    let format = record.format().to_string();
    match record {
        Record::Delivery(delivery) => spelled(&DeliveryLine {
            format,
            key: delivery.key().to_string(),
            readers: delivery.readers().iter().map(Reader::to_string).collect(),
            message: delivery.message().to_string(),
            recorded_at: delivery.recorded_at().to_string(),
            expires_at: delivery.expires_at().to_string(),
        }),
        Record::Handover(handover) => spelled(&HandoverLine {
            format,
            key: handover.key.to_string(),
            reader: handover.reader.to_string(),
            by: handover.by.to_string(),
            at: handover.at.to_string(),
        }),
        Record::Confirmation(confirmation) => spelled(&ConfirmationLine {
            format,
            key: confirmation.key.to_string(),
            reader: confirmation.reader.to_string(),
            at: confirmation.at.to_string(),
        }),
        Record::Known(known) => spelled(&KnownLine {
            format,
            reader: known.reader.to_string(),
            known_at: known.known_at.to_string(),
        }),
    }
}

/// The records as JSON lines, one per record, each ending in a line end.
pub fn write_lines(records: &[Record]) -> String {
    records.iter().map(|record| write(record) + "\n").collect()
}

/// The record one line of JSON holds; or a refusal naming the field: a
/// format the crate does not read, a field missing, one the format does not
/// name, given twice or not of its shape (C.8; core.52).
pub fn read(line: &str) -> Result<Record, Refusal> {
    let object: Map<String, Value> = serde_json::from_str(line)
        .map_err(|error| Refusal::new("line", format!("is no JSON object: {error}")))?;
    let format = match object.get("format") {
        None => return Err(Refusal::new("format", "is missing")),
        Some(Value::String(format)) => format.as_str(),
        Some(_) => return Err(Refusal::new("format", "is not a string")),
    };
    let fields = fields_of(format).ok_or_else(|| {
        Refusal::new("format", format!("`{format}` is no format the crate reads"))
    })?;
    if let Some(name) = object.keys().find(|name| !fields.contains(&name.as_str())) {
        return Err(Refusal::new(name, format!("is no field of `{format}`")));
    }
    for name in fields {
        match (object.get(*name), *name) {
            (None, _) => return Err(Refusal::new(*name, "is missing")),
            (Some(Value::Array(list)), "readers") if list.iter().all(Value::is_string) => {}
            (Some(_), "readers") => return Err(Refusal::new(*name, "is not a list of strings")),
            (Some(Value::String(_)), _) => {}
            (Some(_), _) => return Err(Refusal::new(*name, "is not a string")),
        }
    }
    Ok(match format {
        DELIVERY => {
            let line: DeliveryLine = structured(line)?;
            let readers = line
                .readers
                .iter()
                .map(|reader| Reader::parse_as(reader, "readers"))
                .collect::<Result<Vec<_>, _>>()?;
            Delivery::read(
                Key::parse(&line.key)?,
                readers,
                Message::parse(&line.message)?,
                Time::parse(&line.recorded_at, "recorded_at")?,
                Time::parse(&line.expires_at, "expires_at")?,
                "readers",
            )?
            .into()
        }
        HANDOVER => {
            let line: HandoverLine = structured(line)?;
            Handover {
                key: Key::parse(&line.key)?,
                reader: Reader::parse(&line.reader)?,
                by: Way::parse(&line.by)?,
                at: Time::parse(&line.at, "at")?,
            }
            .into()
        }
        CONFIRMATION => {
            let line: ConfirmationLine = structured(line)?;
            Confirmation {
                key: Key::parse(&line.key)?,
                reader: Reader::parse(&line.reader)?,
                at: Time::parse(&line.at, "at")?,
            }
            .into()
        }
        _ => {
            let line: KnownLine = structured(line)?;
            Known {
                reader: Reader::parse(&line.reader)?,
                known_at: Time::parse(&line.known_at, "known_at")?,
            }
            .into()
        }
    })
}

/// The line read through its format's struct, which refuses a field given
/// twice, naming it.
fn structured<T: for<'de> Deserialize<'de>>(line: &str) -> Result<T, Refusal> {
    serde_json::from_str(line).map_err(|error| {
        let said = error.to_string();
        let field = said
            .strip_prefix("duplicate field `")
            .and_then(|rest| rest.split_once('`'))
            .map_or("line", |(field, _)| field);
        Refusal::new(field, said.clone())
    })
}

/// The records of JSON lines, one per line, an empty line passed over; or
/// the first line's refusal, saying which line.
pub fn read_lines(text: &str) -> Result<Vec<Record>, Refusal> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(n, line)| {
            read(line).map_err(|refused| {
                Refusal::new(
                    refused.field,
                    format!("{}, on line {}", refused.reason, n + 1),
                )
            })
        })
        .collect()
}
