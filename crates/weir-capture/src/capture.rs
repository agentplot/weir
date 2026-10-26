//! The capture's record, `weir-capture/1`: one event as an inbox records
//! it, in recutils (16, 17, 22, 40; C.2; core.22). Its own signal is handed
//! to the inbox beside it, and recorded by `weir-signals`.

use std::fmt;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::key::Key;
use crate::rec::{self, RecError, Record};

/// The format and major every capture's record names (40).
pub const FORMAT: &str = "weir-capture/1";

/// How a capture came in (22).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CameIn {
    /// From a run of the adapter named.
    Run(String),
    Intake,
    /// By an import of the body named.
    Import(String),
}

impl CameIn {
    /// `run:<adapter>`, `intake` or `import:<body>`.
    pub fn parse(text: &str) -> Option<CameIn> {
        if text == "intake" {
            return Some(CameIn::Intake);
        }
        if let Some(adapter) = text.strip_prefix("run:") {
            return (!adapter.is_empty()).then(|| CameIn::Run(adapter.to_string()));
        }
        if let Some(body) = text.strip_prefix("import:") {
            return (!body.is_empty()).then(|| CameIn::Import(body.to_string()));
        }
        None
    }
}

impl fmt::Display for CameIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CameIn::Run(adapter) => write!(f, "run:{adapter}"),
            CameIn::Intake => f.write_str("intake"),
            CameIn::Import(body) => write!(f, "import:{body}"),
        }
    }
}

/// One capture, as its record holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub key: Key,
    pub source: String,
    /// As offered (16; core.8).
    pub event_at: String,
    pub title: Option<String>,
    pub url: Option<String>,
    /// `<machine>:<path>`, only where something was banked (17; core.11).
    pub pointer: Option<String>,
    pub came_in: CameIn,
    /// The machine that took it, by its short hostname (17, 22).
    pub machine: String,
    /// When it was taken, in RFC 3339 in UTC (22).
    pub taken_at: String,
    /// That it has no own signal and no reading (19).
    pub unread: bool,
}

impl Capture {
    /// The record, `format` first, then the fields in C.2's order, each
    /// with no value left out.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        record.push("format", FORMAT);
        record.push("key", self.key.as_str());
        record.push("source", self.source.as_str());
        record.push("event_at", self.event_at.as_str());
        record.push_optional("title", self.title.as_deref());
        record.push_optional("url", self.url.as_deref());
        record.push_optional("pointer", self.pointer.as_deref());
        record.push("came_in", self.came_in.to_string());
        record.push("machine", self.machine.as_str());
        record.push("taken_at", self.taken_at.as_str());
        record.push("unread", if self.unread { "yes" } else { "no" });
        record
    }

    /// The record as `capture.rec` holds it.
    pub fn to_rec(&self) -> String {
        rec::write(&[self.to_record()])
    }

    /// The capture of a `weir-capture/1` record, refused naming the field
    /// that is missing, given twice or not of its shape. A field the format
    /// does not name is ignored.
    pub fn from_record(record: &Record) -> Result<Capture, RecError> {
        let format = record.required("format")?;
        if format != FORMAT {
            return Err(RecError::field(
                "format",
                format!("is `{format}`, not `{FORMAT}`"),
            ));
        }
        let key_text = record.required("key")?;
        let key = Key::parse(key_text)
            .map_err(|error| RecError::field("key", format!("is `{key_text}`: {error}")))?;
        let event_at = time(record, "event_at")?;
        let came_in_text = record.required("came_in")?;
        let came_in = CameIn::parse(came_in_text).ok_or_else(|| {
            RecError::field(
                "came_in",
                format!("is `{came_in_text}`, not run:<adapter>, intake or import:<body>"),
            )
        })?;
        let unread = match record.required("unread")? {
            "yes" => true,
            "no" => false,
            other => {
                return Err(RecError::field(
                    "unread",
                    format!("is `{other}`, not yes or no"),
                ))
            }
        };
        Ok(Capture {
            key,
            source: record.required("source")?.to_string(),
            event_at,
            title: record.single("title")?.map(str::to_string),
            url: record.single("url")?.map(str::to_string),
            pointer: record.single("pointer")?.map(str::to_string),
            came_in,
            machine: record.required("machine")?.to_string(),
            taken_at: time(record, "taken_at")?,
            unread,
        })
    }

    /// The capture `capture.rec` holds: exactly one record.
    pub fn from_rec(text: &str) -> Result<Capture, RecError> {
        match rec::read(text)?.as_slice() {
            [record] => Capture::from_record(record),
            records => Err(RecError::text(format!(
                "a capture's record is one record, and this text holds {}",
                records.len()
            ))),
        }
    }
}

fn time(record: &Record, name: &str) -> Result<String, RecError> {
    let text = record.required(name)?;
    OffsetDateTime::parse(text, &Rfc3339)
        .map(|_| text.to_string())
        .map_err(|_| RecError::field(name, format!("is `{text}`, not an RFC 3339 time")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{recfix, recsel_value};

    fn from_a_run() -> Capture {
        Capture {
            key: Key::parse("release/mise/2026.10.10").unwrap(),
            source: "release".into(),
            event_at: "2026-10-10T14:02:11Z".into(),
            title: Some("mise 2026.10.10".into()),
            url: Some("https://github.com/jdx/mise/releases/tag/v2026.10.10".into()),
            pointer: Some("deneb:/data/inbox/release/mise/2026.10.10".into()),
            came_in: CameIn::Run("releases".into()),
            machine: "deneb".into(),
            taken_at: "2026-10-17T04:30:05Z".into(),
            unread: false,
        }
    }

    #[test]
    fn a_capture_from_a_run() {
        let capture = from_a_run();
        let written = capture.to_rec();
        assert_eq!(
            written,
            "format: weir-capture/1\n\
             key: release/mise/2026.10.10\n\
             source: release\n\
             event_at: 2026-10-10T14:02:11Z\n\
             title: mise 2026.10.10\n\
             url: https://github.com/jdx/mise/releases/tag/v2026.10.10\n\
             pointer: deneb:/data/inbox/release/mise/2026.10.10\n\
             came_in: run:releases\n\
             machine: deneb\n\
             taken_at: 2026-10-17T04:30:05Z\n\
             unread: no\n"
        );
        recfix(&written);
        assert_eq!(Capture::from_rec(&written), Ok(capture));
    }

    #[test]
    fn a_multi_line_value_stays_one_field() {
        let capture = Capture {
            title: Some("mise 2026.10.10\nthe shim fixed".into()),
            ..from_a_run()
        };
        let written = capture.to_rec();
        assert!(
            written.contains("title: mise 2026.10.10\n+ the shim fixed\n"),
            "{written}"
        );
        recfix(&written);
        assert_eq!(
            recsel_value(&written, "title"),
            "mise 2026.10.10\nthe shim fixed"
        );
        assert_eq!(Capture::from_rec(&written), Ok(capture));
    }

    #[test]
    fn a_field_with_no_value_is_left_out() {
        let capture = Capture {
            title: None,
            url: None,
            pointer: None,
            came_in: CameIn::Intake,
            unread: true,
            ..from_a_run()
        };
        let written = capture.to_rec();
        for name in ["title", "url", "pointer"] {
            assert!(!written.contains(&format!("{name}:")), "{written}");
        }
        assert!(written.contains("came_in: intake\n"), "{written}");
        assert!(written.ends_with("unread: yes\n"), "{written}");
        recfix(&written);
        assert_eq!(Capture::from_rec(&written), Ok(capture));
    }

    #[test]
    fn every_way_in_reads_back() {
        for came_in in [
            CameIn::Run("releases".into()),
            CameIn::Intake,
            CameIn::Import("partition-x".into()),
        ] {
            let capture = Capture {
                came_in: came_in.clone(),
                ..from_a_run()
            };
            let written = capture.to_rec();
            recfix(&written);
            assert_eq!(Capture::from_rec(&written), Ok(capture), "{came_in}");
        }
        assert_eq!(CameIn::parse("run:"), None);
        assert_eq!(CameIn::parse("import:"), None);
        assert_eq!(CameIn::parse("cron"), None);
    }

    #[test]
    fn a_record_not_of_the_format_is_refused_naming_the_field() {
        let written = from_a_run().to_rec();
        for (from, to, field) in [
            ("format: weir-capture/1", "format: weir-signal/1", "format"),
            ("key: release/mise/2026.10.10", "key: moves/x", "key"),
            ("source: release\n", "", "source"),
            ("came_in: run:releases", "came_in: cron", "came_in"),
            ("unread: no", "unread: maybe", "unread"),
            (
                "taken_at: 2026-10-17T04:30:05Z",
                "taken_at: yesterday",
                "taken_at",
            ),
            (
                "machine: deneb\n",
                "machine: deneb\nmachine: vega\n",
                "machine",
            ),
        ] {
            let changed = written.replace(from, to);
            let error = Capture::from_rec(&changed).unwrap_err();
            assert_eq!(error.field.as_deref(), Some(field), "{changed}");
        }
        assert!(Capture::from_rec("").is_err());
        assert!(Capture::from_rec(&format!("{written}\n{written}")).is_err());
    }
}
