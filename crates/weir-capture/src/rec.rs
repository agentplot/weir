//! Records at rest, as GNU recutils (C.2; section 9): one `name: value`
//! line per field in the order given, a value's later lines as `+ `
//! continuation lines, a field with no value left out, a list as one field
//! per entry, and records separated by a blank line. Every record a write
//! changes passes `recfix` before it lands (core.24); the crate's tests hold
//! what it writes to `recfix`, and the crate itself never runs it (core.1).
//!
//! recutils joins a line that ends in `\` to the next. A value's line that
//! ends in `\` is written with one more `\` and an empty line after it, so
//! that recutils, and this reader, read back the line as it was.

use std::fmt;

/// One record: its fields in order, a name given as often as its list
/// holds entries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Record {
    fields: Vec<(String, String)>,
}

/// A record that could not be read, naming the field where there is one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecError {
    pub field: Option<String>,
    pub reason: String,
}

impl RecError {
    pub fn field(field: impl Into<String>, reason: impl Into<String>) -> RecError {
        RecError {
            field: Some(field.into()),
            reason: reason.into(),
        }
    }

    pub fn text(reason: impl Into<String>) -> RecError {
        RecError {
            field: None,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for RecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "`{field}` {}", self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for RecError {}

/// A field's name as recutils takes it: a letter or `%`, then letters,
/// digits and `_`.
fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '%')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Record {
    pub fn new() -> Record {
        Record::default()
    }

    /// Adds the field `name` with `value`, which may hold several lines.
    pub fn push(&mut self, name: &str, value: impl Into<String>) {
        assert!(is_name(name), "`{name}` is not a recutils field name");
        self.fields.push((name.to_string(), value.into()));
    }

    /// Adds the field when it has a value, and leaves it out otherwise.
    pub fn push_optional(&mut self, name: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.push(name, value);
        }
    }

    /// Adds one field `name` per entry, none for an empty list.
    pub fn push_list<S: AsRef<str>>(&mut self, name: &str, values: &[S]) {
        for value in values {
            self.push(name, value.as_ref());
        }
    }

    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }

    /// The first value of `name`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.get_all(name).into_iter().next()
    }

    /// Every value of `name`, in order.
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|(field, _)| field == name)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// The one value of `name`, or none; a name given twice is refused.
    pub fn single(&self, name: &str) -> Result<Option<&str>, RecError> {
        match self.get_all(name).as_slice() {
            [] => Ok(None),
            [value] => Ok(Some(value)),
            _ => Err(RecError::field(name, "is given more than once")),
        }
    }

    /// The one value of `name`, refused when it is missing.
    pub fn required(&self, name: &str) -> Result<&str, RecError> {
        self.single(name)?
            .ok_or_else(|| RecError::field(name, "is missing"))
    }
}

/// The records as recutils text.
pub fn write(records: &[Record]) -> String {
    let mut text = String::new();
    for (n, record) in records.iter().enumerate() {
        if n > 0 {
            text.push('\n');
        }
        for (name, value) in &record.fields {
            for (m, line) in value.split('\n').enumerate() {
                match (m, line.is_empty()) {
                    (0, true) => {
                        text.push_str(name);
                        text.push(':');
                    }
                    (0, false) => {
                        text.push_str(name);
                        text.push_str(": ");
                    }
                    (_, true) => text.push('+'),
                    (_, false) => text.push_str("+ "),
                }
                text.push_str(line);
                if line.ends_with('\\') {
                    text.push_str("\\\n");
                }
                text.push('\n');
            }
        }
    }
    text
}

/// The records of recutils text: comment lines skipped, a line ending in
/// `\` joined to the next, a blank line between records.
pub fn read(text: &str) -> Result<Vec<Record>, RecError> {
    let mut records = Vec::new();
    let mut record = Record::new();
    let mut physical = text.split_inclusive('\n').enumerate();
    while let Some((n, first)) = physical.next() {
        let number = n + 1;
        // Only the physical line's own end joins: a line joined to an empty
        // one ends there, whatever the joined text ends in.
        let mut line = String::new();
        let mut piece = first.strip_suffix('\n').unwrap_or(first);
        loop {
            let Some(joined) = piece.strip_suffix('\\') else {
                line.push_str(piece);
                break;
            };
            line.push_str(joined);
            match physical.next() {
                Some((_, next)) => piece = next.strip_suffix('\n').unwrap_or(next),
                None => break,
            }
        }
        if line.trim().is_empty() {
            if !record.fields.is_empty() {
                records.push(std::mem::take(&mut record));
            }
        } else if line.starts_with('#') {
            continue;
        } else if let Some(rest) = line.strip_prefix('+') {
            let Some((_, value)) = record.fields.last_mut() else {
                return Err(RecError::text(format!("line {number} continues no field")));
            };
            value.push('\n');
            value.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        } else {
            let Some((name, value)) = line.split_once(':') else {
                return Err(RecError::text(format!(
                    "line {number} is neither a field, a continuation nor a comment"
                )));
            };
            if !is_name(name) {
                return Err(RecError::text(format!(
                    "line {number} names `{name}`, which is not a field name"
                )));
            }
            let value = value
                .strip_prefix(' ')
                .or_else(|| value.strip_prefix('\t'))
                .unwrap_or(value);
            record.fields.push((name.to_string(), value.to_string()));
        }
    }
    if !record.fields.is_empty() {
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{recfix, recsel_value};

    fn record(fields: &[(&str, &str)]) -> Record {
        let mut record = Record::new();
        for (name, value) in fields {
            record.push(name, *value);
        }
        record
    }

    #[test]
    fn a_record_is_one_line_per_field_in_order() {
        let mut capture = record(&[("format", "weir-capture/1"), ("key", "release/x")]);
        capture.push_optional("title", None);
        capture.push_optional("url", Some("https://example.org/x"));
        assert_eq!(
            write(&[capture]),
            "format: weir-capture/1\nkey: release/x\nurl: https://example.org/x\n"
        );
    }

    #[test]
    fn a_multi_line_value_is_continued_and_read_back() {
        let value = "## Fixes\n\n- the shim\n  indented\n+ a plus\n# a hash\n";
        let written = write(&[record(&[("format", "x/1"), ("excerpt", value)])]);
        assert_eq!(
            written,
            "format: x/1\nexcerpt: ## Fixes\n+\n+ - the shim\n+   indented\n+ + a plus\n+ # a hash\n+\n"
        );
        recfix(&written);
        assert_eq!(recsel_value(&written, "excerpt"), value);
        assert_eq!(read(&written).unwrap()[0].get("excerpt"), Some(value));
    }

    #[test]
    fn an_empty_list_writes_no_field_and_reads_back_empty() {
        let mut signal = record(&[("signal", "release/x#1")]);
        signal.push_list::<String>("argues_with", &[]);
        signal.push_list("subjects", &["release", "mise"]);
        let written = write(&[signal.clone()]);
        assert_eq!(
            written,
            "signal: release/x#1\nsubjects: release\nsubjects: mise\n"
        );
        recfix(&written);
        let read = read(&written).unwrap();
        assert_eq!(read, [signal]);
        assert!(read[0].get_all("argues_with").is_empty());
        assert_eq!(read[0].get_all("subjects"), ["release", "mise"]);
    }

    #[test]
    fn records_are_separated_by_a_blank_line() {
        let records = [
            record(&[("move", "drop"), ("target", "noise")]),
            record(&[("move", "join"), ("target", "items-x")]),
        ];
        let written = write(&records);
        assert_eq!(
            written,
            "move: drop\ntarget: noise\n\nmove: join\ntarget: items-x\n"
        );
        recfix(&written);
        assert_eq!(read(&written).unwrap(), records);
        assert_eq!(read("").unwrap(), []);
    }

    #[test]
    fn edges_of_a_value_read_back_as_written() {
        for value in [
            "",
            " a leading space",
            "\na first line empty",
            "a last line empty\n",
            "ends in a backslash\\",
            "C:\\path\\\nand on",
            "two\\\\\n\\",
            "tab\tinside",
        ] {
            let written = write(&[record(&[
                ("format", "x/1"),
                ("value", value),
                ("after", "y"),
            ])]);
            recfix(&written);
            assert_eq!(recsel_value(&written, "value"), value, "{written}");
            assert_eq!(recsel_value(&written, "after"), "y", "{written}");
            let read = read(&written).unwrap();
            assert_eq!(read.len(), 1, "{written}");
            assert_eq!(read[0].get("value"), Some(value), "{written}");
            assert_eq!(read[0].get("after"), Some("y"), "{written}");
        }
    }

    #[test]
    fn a_comment_is_skipped_and_a_stray_line_refused() {
        let read = read("# a comment\nkey: a\n+ b\n").unwrap();
        assert_eq!(read[0].get("key"), Some("a\nb"));
        assert!(super::read("+ nothing to continue\n").is_err());
        assert!(super::read("no colon here\n").is_err());
        assert!(super::read("1name: x\n").is_err());
    }

    #[test]
    fn a_single_value_is_refused_when_given_twice() {
        let twice = record(&[("key", "a"), ("key", "b")]);
        assert_eq!(
            twice.single("key").unwrap_err().to_string(),
            "`key` is given more than once"
        );
        assert_eq!(
            twice.required("source").unwrap_err().to_string(),
            "`source` is missing"
        );
    }
}
