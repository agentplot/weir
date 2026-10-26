//! A record's format, `weir-<record>/<major>`, read and written (40; C.2;
//! core.22). A reader reads every major up to the latest it knows, and
//! refuses a later one, so it never writes into an inbox it could not read.
//! Every format is at its first major.

use std::fmt;

use weir_capture::rec::{RecError, Record};

/// The latest major this crate reads, and the one it writes.
pub const MAJOR: u32 = 1;

/// The records an inbox holds, each by the name its format gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Format {
    Capture,
    Signal,
    Move,
    Batch,
    Item,
    Send,
    Brief,
}

/// Why a format was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatError {
    /// Not of the shape `weir-<record>/<major>`, or a record no inbox holds.
    NotAFormat(String),
    /// A format other than one the place holds.
    Elsewhere { holds: String, lies: Vec<Format> },
    /// A major later than [`MAJOR`].
    Later(String),
}

impl Format {
    pub const ALL: [Format; 7] = [
        Format::Capture,
        Format::Signal,
        Format::Move,
        Format::Batch,
        Format::Item,
        Format::Send,
        Format::Brief,
    ];

    /// The record's name in its format.
    pub fn name(self) -> &'static str {
        match self {
            Format::Capture => "capture",
            Format::Signal => "signal",
            Format::Move => "move",
            Format::Batch => "batch",
            Format::Item => "item",
            Format::Send => "send",
            Format::Brief => "brief",
        }
    }

    /// The format and major this crate writes: `weir-<record>/1`.
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Capture => "weir-capture/1",
            Format::Signal => "weir-signal/1",
            Format::Move => "weir-move/1",
            Format::Batch => "weir-batch/1",
            Format::Item => "weir-item/1",
            Format::Send => "weir-send/1",
            Format::Brief => "weir-brief/1",
        }
    }

    /// The format `text` names, one of `lies`, at a major this crate reads.
    pub fn read(text: &str, lies: &[Format]) -> Result<Format, FormatError> {
        let not_a_format = || FormatError::NotAFormat(text.to_string());
        let (name, major_text) = text
            .strip_prefix("weir-")
            .and_then(|rest| rest.split_once('/'))
            .ok_or_else(not_a_format)?;
        let format = Format::ALL
            .into_iter()
            .find(|format| format.name() == name)
            .ok_or_else(not_a_format)?;
        // A major is a count from 1, written in decimal with no sign and no
        // leading zero.
        let major = major_text
            .parse::<u32>()
            .ok()
            .filter(|major| *major >= 1 && major.to_string() == major_text)
            .ok_or_else(not_a_format)?;
        if !lies.contains(&format) {
            return Err(FormatError::Elsewhere {
                holds: text.to_string(),
                lies: lies.to_vec(),
            });
        }
        if major > MAJOR {
            return Err(FormatError::Later(text.to_string()));
        }
        Ok(format)
    }

    /// The format `record` names in its field `format`, one of `lies`,
    /// refused naming the field.
    pub fn of_record(record: &Record, lies: &[Format]) -> Result<Format, RecError> {
        let text = record.required("format")?;
        Format::read(text, lies).map_err(|error| RecError::field("format", error.to_string()))
    }

    /// The line a brief opens with, so it names its format without showing
    /// it to the person who reads the rendered brief (40).
    pub fn brief_line() -> String {
        format!("<!-- format: {} -->", Format::Brief.as_str())
    }

    /// The text of `brief`, after the line naming its format; refused when
    /// its first line names none, or another format, or a later major.
    pub fn read_brief(brief: &str) -> Result<&str, String> {
        let (first, text) = brief.split_once('\n').unwrap_or((brief, ""));
        let named = first
            .strip_prefix("<!-- format: ")
            .and_then(|rest| rest.strip_suffix(" -->"))
            .ok_or_else(|| {
                format!(
                    "a brief's first line names its format, `{}`, and this one is `{first}`",
                    Format::brief_line()
                )
            })?;
        Format::read(named, &[Format::Brief]).map_err(|error| format!("its format {error}"))?;
        Ok(text)
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::NotAFormat(text) => write!(
                f,
                "is `{text}`, not a format an inbox holds, `weir-<record>/<major>`"
            ),
            FormatError::Elsewhere { holds, lies } => {
                let lies: Vec<String> = lies.iter().map(|format| format!("`{format}`")).collect();
                write!(f, "is `{holds}`, where {} lies", lies.join(" or "))
            }
            FormatError::Later(text) => write!(
                f,
                "is `{text}`, a later major than this reader reads, which is {MAJOR}"
            ),
        }
    }
}

impl std::error::Error for FormatError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_format_reads_back_as_written() {
        for format in Format::ALL {
            assert_eq!(Format::read(format.as_str(), &[format]), Ok(format));
            assert_eq!(
                format.as_str(),
                format!("weir-{}/{MAJOR}", format.name()),
                "{format:?}"
            );
        }
    }

    #[test]
    fn a_later_major() {
        let error = Format::read("weir-move/2", &[Format::Move]).unwrap_err();
        assert_eq!(error, FormatError::Later("weir-move/2".into()));
        assert_eq!(
            error.to_string(),
            "is `weir-move/2`, a later major than this reader reads, which is 1"
        );
    }

    #[test]
    fn a_record_of_another_format_where_it_lies() {
        let error = Format::read("weir-move/1", &[Format::Send]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "is `weir-move/1`, where `weir-send/1` lies"
        );
        let error = Format::read("weir-item/1", &[Format::Batch, Format::Move]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "is `weir-item/1`, where `weir-batch/1` or `weir-move/1` lies"
        );
    }

    #[test]
    fn a_text_that_is_no_format() {
        for text in [
            "",
            "weir-move",
            "move/1",
            "weir-move/0",
            "weir-move/01",
            "weir-move/+1",
            "weir-move/one",
            "weir-chore/1",
            "weir-move/1/2",
        ] {
            assert!(
                matches!(
                    Format::read(text, &[Format::Move]),
                    Err(FormatError::NotAFormat(_))
                ),
                "{text}"
            );
        }
    }

    #[test]
    fn a_record_names_its_format_in_its_field() {
        let mut record = Record::new();
        record.push("format", "weir-move/1");
        assert_eq!(
            Format::of_record(&record, &[Format::Move]),
            Ok(Format::Move)
        );
        let error = Format::of_record(&Record::new(), &[Format::Move]).unwrap_err();
        assert_eq!(error.field.as_deref(), Some("format"));
        assert_eq!(error.to_string(), "`format` is missing");
    }

    #[test]
    fn a_brief_names_its_format_on_its_first_line() {
        assert_eq!(Format::brief_line(), "<!-- format: weir-brief/1 -->");
        assert_eq!(
            Format::read_brief("<!-- format: weir-brief/1 -->\n# Shim\n\nthe text\n"),
            Ok("# Shim\n\nthe text\n")
        );
        assert_eq!(Format::read_brief("<!-- format: weir-brief/1 -->"), Ok(""));
        assert!(Format::read_brief("# Shim\n")
            .unwrap_err()
            .contains("names its format"));
        assert!(Format::read_brief("<!-- format: weir-brief/2 -->\n")
            .unwrap_err()
            .contains("a later major"));
    }
}
