//! A delivery's four records (C.8), each written once and never changed:
//! what happens to a delivery is added beside it (100, 41). Each names its
//! format (40; core.22).

use crate::calls::Refusal;
use crate::text::{Key, Message, Reader, Span, Time, Way};

/// `weir-delivery/1`: one message a deliverer owes its readers, under its
/// key, with when it was recorded and when it expires (100, 104).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Delivery {
    key: Key,
    readers: Vec<Reader>,
    message: Message,
    recorded_at: Time,
    expires_at: Time,
}

/// `weir-handover/1`: a delivery handed to one of its readers, the way its
/// deliverer names, and when; it confirms nothing (103).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Handover {
    /// The delivery's key.
    pub key: Key,
    /// The reader it was handed to.
    pub reader: Reader,
    /// The way, as its deliverer names it.
    pub by: Way,
    /// When.
    pub at: Time,
}

/// `weir-confirmation/1`: a reader's confirmation of a delivery, naming its
/// key (102).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Confirmation {
    /// The delivery's key.
    pub key: Key,
    /// The reader that confirmed it.
    pub reader: Reader,
    /// When.
    pub at: Time,
}

/// `weir-known/1`: a name a reader has confirmed under, and when it first
/// did (106).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Known {
    /// The name.
    pub reader: Reader,
    /// When a reader first confirmed under it.
    pub known_at: Time,
}

/// One of a delivery's records (C.8).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Record {
    /// `weir-delivery/1`.
    Delivery(Delivery),
    /// `weir-handover/1`.
    Handover(Handover),
    /// `weir-confirmation/1`.
    Confirmation(Confirmation),
    /// `weir-known/1`.
    Known(Known),
}

/// The format of a delivery.
pub const DELIVERY: &str = "weir-delivery/1";
/// The format of a hand-over.
pub const HANDOVER: &str = "weir-handover/1";
/// The format of a confirmation.
pub const CONFIRMATION: &str = "weir-confirmation/1";
/// The format of a known name.
pub const KNOWN: &str = "weir-known/1";

/// `readers` as a delivery holds them: at least one, none named twice, or a
/// refusal naming `field` (100).
pub(crate) fn readers(readers: Vec<Reader>, field: &str) -> Result<Vec<Reader>, Refusal> {
    if readers.is_empty() {
        return Err(Refusal::new(field, "names no reader"));
    }
    for (n, reader) in readers.iter().enumerate() {
        if readers[..n].contains(reader) {
            return Err(Refusal::new(field, format!("names `{reader}` twice")));
        }
    }
    Ok(readers)
}

impl Delivery {
    /// The delivery of `message` to `readers` under `key`, recorded at
    /// `recorded_at` and expiring `span` after it, at its offset (100,
    /// 104); or a refusal naming `readers` when it names no reader or one
    /// twice, or `span` when the expiry lies past every time RFC 3339
    /// writes.
    pub fn new(
        key: Key,
        message: Message,
        readers: Vec<Reader>,
        recorded_at: Time,
        span: Span,
    ) -> Result<Delivery, Refusal> {
        let readers = self::readers(readers, "readers")?;
        let expires_at = recorded_at.after(span)?;
        Ok(Delivery {
            key,
            readers,
            message,
            recorded_at,
            expires_at,
        })
    }

    /// A delivery as a spelling read it, its expiry as written; or a
    /// refusal naming `readers_field` (`reader` or `readers`) when it names
    /// no reader or one twice.
    pub(crate) fn read(
        key: Key,
        readers: Vec<Reader>,
        message: Message,
        recorded_at: Time,
        expires_at: Time,
        readers_field: &str,
    ) -> Result<Delivery, Refusal> {
        Ok(Delivery {
            key,
            readers: self::readers(readers, readers_field)?,
            message,
            recorded_at,
            expires_at,
        })
    }

    /// Its key.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Its readers, in the order given.
    pub fn readers(&self) -> &[Reader] {
        &self.readers
    }

    /// Its message, as given.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// When it was recorded.
    pub fn recorded_at(&self) -> &Time {
        &self.recorded_at
    }

    /// When it expires, fixed when it was recorded (104).
    pub fn expires_at(&self) -> &Time {
        &self.expires_at
    }
}

impl Record {
    /// The format it names: `weir-delivery/1`, `weir-handover/1`,
    /// `weir-confirmation/1` or `weir-known/1` (40; C.8).
    pub fn format(&self) -> &'static str {
        match self {
            Record::Delivery(_) => DELIVERY,
            Record::Handover(_) => HANDOVER,
            Record::Confirmation(_) => CONFIRMATION,
            Record::Known(_) => KNOWN,
        }
    }
}

impl From<Delivery> for Record {
    fn from(delivery: Delivery) -> Record {
        Record::Delivery(delivery)
    }
}

impl From<Handover> for Record {
    fn from(handover: Handover) -> Record {
        Record::Handover(handover)
    }
}

impl From<Confirmation> for Record {
    fn from(confirmation: Confirmation) -> Record {
        Record::Confirmation(confirmation)
    }
}

impl From<Known> for Record {
    fn from(known: Known) -> Record {
        Record::Known(known)
    }
}
