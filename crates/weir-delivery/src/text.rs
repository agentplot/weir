//! The values a delivery's records hold (C.8): a key, a reader and a way,
//! each one line; a message of one or more lines; a time, RFC 3339 with its
//! offset and kept as written; and a span, a whole number of seconds. Each
//! is built only by parsing, which refuses any other text naming the field
//! it was read for (core.47), so every value is spelled the same in
//! recutils and in JSON.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::calls::Refusal;

/// `text` as one line of a field: never empty, holding no line break, and
/// never beginning or ending with a space or a tab (C.8).
fn line(text: &str, field: &str, what: &str) -> Result<String, Refusal> {
    if text.is_empty() {
        return Err(Refusal::new(field, "is empty"));
    }
    if text.contains(['\n', '\r']) {
        return Err(Refusal::new(
            field,
            format!("holds a line break, and {what} is one line"),
        ));
    }
    if text.starts_with([' ', '\t']) || text.ends_with([' ', '\t']) {
        return Err(Refusal::new(
            field,
            format!("`{text}` begins or ends with a space or a tab"),
        ));
    }
    Ok(text.to_string())
}

macro_rules! one_line {
    ($(#[$doc:meta])* $name:ident, $field:literal, $what:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("The ", $what, " `text` names, or a refusal naming `", $field, "`.")]
            pub fn parse(text: &str) -> Result<$name, Refusal> {
                $name::parse_as(text, $field)
            }

            /// As `parse`, a refusal naming `field`, the argument or the
            /// field it was given as.
            pub(crate) fn parse_as(text: &str, field: &str) -> Result<$name, Refusal> {
                line(text, field, $what).map($name)
            }

            /// The text, as given.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

one_line!(
    /// The key a deliverer gives a delivery, naming it alone (100, 101).
    Key,
    "key",
    "a key"
);

one_line!(
    /// A reader, named as its deliverer names it (106), never folded in
    /// case or matched loosely (core.50).
    Reader,
    "reader",
    "a reader"
);

one_line!(
    /// The way a delivery was handed over, as its deliverer names it (103).
    Way,
    "by",
    "a way"
);

/// A delivery's message: text of one or more lines, never empty, kept as
/// given (100). weir reads none of its words (core.4).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message(String);

impl Message {
    /// The message `text` holds, or a refusal naming `message` when it is
    /// empty.
    pub fn parse(text: &str) -> Result<Message, Refusal> {
        if text.is_empty() {
            return Err(Refusal::new("message", "is empty"));
        }
        Ok(Message(text.to_string()))
    }

    /// The text, as given.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A time, RFC 3339 with its offset, kept as written (C.8). Two times are
/// equal when their text is, as a capture's event time is kept (core.14);
/// they are ordered by their instants, and by their text where the
/// instants are one.
#[derive(Clone, Debug)]
pub struct Time {
    text: String,
    at: OffsetDateTime,
}

impl Time {
    /// The time `text` writes, or a refusal naming `field` when it is not
    /// RFC 3339 with its offset.
    pub fn parse(text: &str, field: &str) -> Result<Time, Refusal> {
        OffsetDateTime::parse(text, &Rfc3339)
            .map(|at| Time {
                text: text.to_string(),
                at,
            })
            .map_err(|_| {
                Refusal::new(
                    field,
                    format!("`{text}` is not an RFC 3339 time with its offset"),
                )
            })
    }

    /// The time `span` after this one, at this one's offset, written by
    /// RFC 3339 (104); or a refusal naming `span` when it lies past every
    /// time RFC 3339 writes.
    pub(crate) fn after(&self, span: Span) -> Result<Time, Refusal> {
        let past = || Refusal::new("span", "takes the time past every time RFC 3339 writes");
        let seconds = i64::try_from(span.as_seconds()).map_err(|_| past())?;
        let at = self
            .at
            .checked_add(time::Duration::seconds(seconds))
            .ok_or_else(past)?;
        let text = at.format(&Rfc3339).map_err(|_| past())?;
        Ok(Time { text, at })
    }

    /// The text, as written.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The instant it names.
    pub fn instant(&self) -> OffsetDateTime {
        self.at
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Time) -> bool {
        self.text == other.text
    }
}

impl Eq for Time {}

impl Hash for Time {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Time) -> Ordering {
        self.at
            .cmp(&other.at)
            .then_with(|| self.text.cmp(&other.text))
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Time) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// The span a deliverer sets for a delivery to expire after: a whole number
/// of seconds, at least one (104; C.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span(u64);

impl Span {
    /// A span of `seconds`, or a refusal naming `span` when it is 0.
    pub fn seconds(seconds: u64) -> Result<Span, Refusal> {
        if seconds == 0 {
            return Err(Refusal::new("span", "is 0 seconds"));
        }
        Ok(Span(seconds))
    }

    /// Its seconds.
    pub fn as_seconds(self) -> u64 {
        self.0
    }
}
