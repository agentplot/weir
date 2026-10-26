//! An adapter's lines, JSON Lines on standard output: each exactly one of
//! an offer, a cursor or a completion (1; C.1; core.5), and never an import
//! line, which a host's import operation alone reads (63, 64). A line is read to a
//! JSON value first and each field checked by name, so a refusal names the
//! field, and a field the contract does not name is ignored (core.6). A
//! line is written as one line of compact JSON, which reads back the same
//! (C.1; core.34).

use std::fmt;

use serde::Serialize;
use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::key::Key;

/// One line an adapter tells its host (1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Line {
    #[serde(rename = "capture")]
    Offer(Offer),
    /// Opaque: kept as given, and never parsed or compared (8; core.9).
    #[serde(rename = "cursor")]
    Cursor(String),
    #[serde(rename = "done")]
    Done(Completion),
}

/// What is handed to a host for one event (2; C.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Offer {
    pub key: Key,
    pub source: String,
    /// The time at the source, in RFC 3339 with its offset, as offered
    /// (5; core.8).
    pub event_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Paths relative to the raw place, none with a `..` segment (6).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub raw: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<Excerpt>,
}

/// An own excerpt: every field of a signal (7, 25).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Excerpt {
    /// Verbatim and whole, never empty.
    pub text: String,
    pub kind: Nature,
    pub asserted_by: String,
    pub assertion: String,
    pub subjects: Vec<String>,
    pub argues_with: Vec<String>,
    pub position: String,
}

/// Which of five a signal is, spelled in the field `kind` (25; section 10,
/// question 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Nature {
    Constraint,
    Ask,
    Question,
    Commitment,
    Reaction,
}

/// The mark a complete run ends with (9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Completion {
    pub listed: u64,
    pub failed: Vec<Failure>,
}

/// One event the listing could not offer, and why (9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Failure {
    pub what: String,
    pub why: String,
}

/// A line refused, with the field it fails on where there is one, and the
/// key it offers where it holds one as a string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Malformed {
    pub field: Option<String>,
    pub reason: String,
    pub key: Option<String>,
}

impl Nature {
    pub const ALL: [Nature; 5] = [
        Nature::Constraint,
        Nature::Ask,
        Nature::Question,
        Nature::Commitment,
        Nature::Reaction,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Nature::Constraint => "constraint",
            Nature::Ask => "ask",
            Nature::Question => "question",
            Nature::Commitment => "commitment",
            Nature::Reaction => "reaction",
        }
    }

    pub fn parse(text: &str) -> Option<Nature> {
        Nature::ALL
            .into_iter()
            .find(|nature| nature.as_str() == text)
    }
}

impl fmt::Display for Nature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Line {
    /// The line as an adapter prints it: one line of JSON, every newline
    /// inside a value escaped, with no newline at its end.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a line is always JSON")
    }
}

impl Malformed {
    pub(crate) fn line(reason: impl Into<String>) -> Malformed {
        Malformed {
            field: None,
            reason: reason.into(),
            key: None,
        }
    }

    pub(crate) fn field(field: impl Into<String>, reason: impl Into<String>) -> Malformed {
        Malformed {
            field: Some(field.into()),
            reason: reason.into(),
            key: None,
        }
    }
}

impl fmt::Display for Malformed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "`{field}` {}", self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for Malformed {}

const LINE_KEYS: [&str; 3] = ["capture", "cursor", "done"];

/// The line `text` is, or why it is malformed.
pub fn read_line(text: &str) -> Result<Line, Malformed> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| Malformed::line(format!("the line is not JSON: {error}")))?;
    let Value::Object(object) = value else {
        return Err(Malformed::line("the line is not a JSON object"));
    };
    if object.contains_key("import") {
        return Err(Malformed::line(
            "the line holds `import`: an import line is read by a host's import operation and \
             never taken (63, 64)",
        ));
    }
    let held: Vec<&str> = LINE_KEYS
        .into_iter()
        .filter(|name| object.contains_key(*name))
        .collect();
    match held.as_slice() {
        [] => Err(Malformed::line(
            "the line holds none of `capture`, `cursor` and `done`",
        )),
        ["capture"] => read_offer(&object["capture"]).map(Line::Offer),
        ["cursor"] => match &object["cursor"] {
            Value::String(cursor) => Ok(Line::Cursor(cursor.clone())),
            _ => Err(Malformed::field("cursor", "is not a string")),
        },
        ["done"] => read_completion(&object["done"]).map(Line::Done),
        several => Err(Malformed::line(format!(
            "the line holds more than one of `capture`, `cursor` and `done`: {}",
            several
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(" and ")
        ))),
    }
}

fn read_offer(value: &Value) -> Result<Offer, Malformed> {
    let Value::Object(offer) = value else {
        return Err(Malformed::field("capture", "is not an object"));
    };
    let key_text = offer.get("key").and_then(Value::as_str).map(str::to_string);
    offer_fields(offer).map_err(|mut malformed| {
        malformed.key = key_text;
        malformed
    })
}

fn offer_fields(offer: &Map<String, Value>) -> Result<Offer, Malformed> {
    let key = read_key(offer, "key")?;
    let source = read_source(offer, "source")?;
    let event_at = read_event_at(offer, "event_at")?;
    let title = string(offer, "title", "title")?;
    let url = string(offer, "url", "url")?;
    let raw = match offer.get("raw") {
        None | Some(Value::Null) => Vec::new(),
        Some(value) => strings(value, "raw")?,
    };
    for (n, path) in raw.iter().enumerate() {
        check_raw_path(path).map_err(|reason| Malformed::field(format!("raw[{n}]"), reason))?;
    }
    let excerpt = match offer.get("excerpt") {
        None | Some(Value::Null) => None,
        Some(value) => Some(read_excerpt(value)?),
    };
    Ok(Offer {
        key,
        source,
        event_at,
        title,
        url,
        raw,
        excerpt,
    })
}

/// A raw path names a file inside the raw place: relative, and with no
/// `..` segment (6). Where it resolves on disk is the host's to check.
fn check_raw_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("is empty".to_string());
    }
    if path.starts_with('/') {
        return Err(format!(
            "is `{path}`, which is not relative to the raw place"
        ));
    }
    if path.split('/').any(|segment| segment == "..") {
        return Err(format!("is `{path}`, which has a `..` segment"));
    }
    Ok(())
}

fn read_excerpt(value: &Value) -> Result<Excerpt, Malformed> {
    let Value::Object(excerpt) = value else {
        return Err(Malformed::field("excerpt", "is not an object"));
    };
    let text = required_string(excerpt, "text", "excerpt.text")?;
    if text.is_empty() {
        return Err(Malformed::field("excerpt.text", "is empty"));
    }
    let kind = read_nature(excerpt, "kind", "excerpt.kind")?;
    let asserted_by = required_string(excerpt, "asserted_by", "excerpt.asserted_by")?;
    let assertion = required_string(excerpt, "assertion", "excerpt.assertion")?;
    let subjects = required(excerpt, "subjects", "excerpt.subjects")
        .and_then(|value| strings(value, "excerpt.subjects"))?;
    let argues_with = required(excerpt, "argues_with", "excerpt.argues_with")
        .and_then(|value| strings(value, "excerpt.argues_with"))?;
    let position = required_string(excerpt, "position", "excerpt.position")?;
    Ok(Excerpt {
        text,
        kind,
        asserted_by,
        assertion,
        subjects,
        argues_with,
        position,
    })
}

fn read_completion(value: &Value) -> Result<Completion, Malformed> {
    let Value::Object(done) = value else {
        return Err(Malformed::field("done", "is not an object"));
    };
    let listed = required(done, "listed", "done.listed")?
        .as_u64()
        .ok_or_else(|| Malformed::field("done.listed", "is not a count"))?;
    let Value::Array(entries) = required(done, "failed", "done.failed")? else {
        return Err(Malformed::field("done.failed", "is not a list"));
    };
    let mut failed = Vec::with_capacity(entries.len());
    for (n, entry) in entries.iter().enumerate() {
        let path = format!("done.failed[{n}]");
        let Value::Object(entry) = entry else {
            return Err(Malformed::field(path, "is not an object"));
        };
        failed.push(Failure {
            what: required_string(entry, "what", &format!("{path}.what"))?,
            why: required_string(entry, "why", &format!("{path}.why"))?,
        });
    }
    Ok(Completion { listed, failed })
}

/// The key the field `key` holds, by the key rule (3, 4); `path` names the
/// field in a refusal, as `key` or `import.capture.key`.
pub(crate) fn read_key(object: &Map<String, Value>, path: &str) -> Result<Key, Malformed> {
    let key_text = required_string(object, "key", path)?;
    Key::parse(&key_text)
        .map_err(|error| Malformed::field(path, format!("is `{key_text}`: {error}")))
}

/// The source the field `source` names, never empty (2).
pub(crate) fn read_source(object: &Map<String, Value>, path: &str) -> Result<String, Malformed> {
    let source = required_string(object, "source", path)?;
    if source.is_empty() {
        return Err(Malformed::field(path, "is empty"));
    }
    Ok(source)
}

/// The event time the field `event_at` holds, in RFC 3339 with its offset,
/// kept as written (5; core.8).
pub(crate) fn read_event_at(object: &Map<String, Value>, path: &str) -> Result<String, Malformed> {
    let event_at = required_string(object, "event_at", path)?;
    if OffsetDateTime::parse(&event_at, &Rfc3339).is_err() {
        return Err(Malformed::field(
            path,
            format!("is `{event_at}`, not an RFC 3339 time with an offset"),
        ));
    }
    Ok(event_at)
}

/// The nature the field `name` spells, one of the five (25).
pub(crate) fn read_nature(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Nature, Malformed> {
    let kind_text = required_string(object, name, path)?;
    Nature::parse(&kind_text).ok_or_else(|| {
        Malformed::field(
            path,
            format!(
                "is `{kind_text}`, not one of {}",
                Nature::ALL.map(Nature::as_str).join(", ")
            ),
        )
    })
}

pub(crate) fn required<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<&'a Value, Malformed> {
    match object.get(name) {
        None | Some(Value::Null) => Err(Malformed::field(path, "is missing")),
        Some(value) => Ok(value),
    }
}

pub(crate) fn required_string(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<String, Malformed> {
    string(object, name, path)?.ok_or_else(|| Malformed::field(path, "is missing"))
}

/// An optional string: absent or `null` is none.
pub(crate) fn string(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<String>, Malformed> {
    match object.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(Malformed::field(path, "is not a string")),
    }
}

pub(crate) fn strings(value: &Value, path: &str) -> Result<Vec<String>, Malformed> {
    let Value::Array(items) = value else {
        return Err(Malformed::field(path, "is not a list of strings"));
    };
    items
        .iter()
        .enumerate()
        .map(|(n, item)| match item {
            Value::String(text) => Ok(text.clone()),
            _ => Err(Malformed::field(format!("{path}[{n}]"), "is not a string")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXCERPT: &str = r#"{"text": "mise 2026.10.10 is out.\nIt fixes the shim.", "kind": "ask", "asserted_by": "jdx/mise", "assertion": "mise 2026.10.10 is released.", "subjects": ["release", "mise"], "argues_with": ["packages/mise/default.nix"], "position": "release notes"}"#;

    fn offer_line(fields: &str) -> String {
        format!(r#"{{"capture": {{{fields}}}}}"#)
    }

    fn refused(line: &str) -> Malformed {
        read_line(line).expect_err(line)
    }

    #[test]
    fn one_key_of_three_reads_as_that_line() {
        let lines = [
            offer_line(
                r#""key": "release/mise/2026.10.10", "source": "release", "event_at": "2026-10-10T14:02:11Z""#,
            ),
            r#"{"cursor": "2026-10-17T04:30:05Z"}"#.to_string(),
            r#"{"done": {"listed": 14, "failed": []}}"#.to_string(),
        ];
        let read: Vec<Line> = lines.iter().map(|line| read_line(line).unwrap()).collect();
        let Line::Offer(offer) = &read[0] else {
            panic!("not an offer: {:?}", read[0]);
        };
        assert_eq!(offer.key.as_str(), "release/mise/2026.10.10");
        assert_eq!(offer.source, "release");
        assert_eq!(offer.title, None);
        assert!(offer.raw.is_empty());
        assert_eq!(offer.excerpt, None);
        assert_eq!(read[1], Line::Cursor("2026-10-17T04:30:05Z".into()));
        assert_eq!(
            read[2],
            Line::Done(Completion {
                listed: 14,
                failed: vec![]
            })
        );
    }

    #[test]
    fn a_line_with_none_or_two_of_the_keys_is_malformed() {
        let two = refused(r#"{"cursor": "a", "done": {"listed": 0, "failed": []}}"#);
        assert_eq!(two.field, None);
        assert!(two.reason.contains("more than one"), "{two}");
        assert!(two.reason.contains("`cursor` and `done`"), "{two}");

        let none = refused(r#"{"note": 1}"#);
        assert!(none.reason.contains("none of"), "{none}");

        let not_json = refused("not json");
        assert!(
            not_json.reason.starts_with("the line is not JSON"),
            "{not_json}"
        );

        let not_object = refused(r#"["cursor"]"#);
        assert_eq!(not_object.reason, "the line is not a JSON object");
    }

    #[test]
    fn an_import_line_is_not_an_adapters_line() {
        let import = r#"{"import": {"capture": {"key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11Z"}, "came_in": "import:crew-folders", "signals": [], "moves": []}}"#;
        let with_offer = offer_line(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11Z""#,
        )
        .replacen('{', r#"{"import": {}, "#, 1);
        for line in [import.to_string(), with_offer] {
            let malformed = refused(&line);
            assert_eq!(malformed.field, None, "{line}");
            assert!(
                malformed
                    .reason
                    .contains("an import line is read by a host's import operation"),
                "{malformed}"
            );
            assert!(malformed.reason.contains("never taken"), "{malformed}");
        }
    }

    #[test]
    fn an_unknown_top_level_key_is_ignored() {
        assert_eq!(
            read_line(r#"{"cursor": "c1", "trace": "x"}"#),
            Ok(Line::Cursor("c1".into()))
        );
    }

    #[test]
    fn a_missing_required_field_refuses_the_offer() {
        let missing = refused(&offer_line(r#""key": "release/x", "source": "release""#));
        assert_eq!(missing.field.as_deref(), Some("event_at"));
        assert_eq!(missing.to_string(), "`event_at` is missing");
        assert_eq!(missing.key.as_deref(), Some("release/x"));

        let no_key = refused(&offer_line(
            r#""source": "release", "event_at": "2026-10-10T14:02:11Z""#,
        ));
        assert_eq!(no_key.field.as_deref(), Some("key"));
        assert_eq!(no_key.key, None);

        let no_source = refused(&offer_line(
            r#""key": "release/x", "event_at": "2026-10-10T14:02:11Z""#,
        ));
        assert_eq!(no_source.field.as_deref(), Some("source"));
    }

    #[test]
    fn an_event_time_without_its_offset_refuses_the_offer() {
        let malformed = refused(&offer_line(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11""#,
        ));
        assert_eq!(malformed.field.as_deref(), Some("event_at"));
        assert!(
            malformed
                .reason
                .contains("not an RFC 3339 time with an offset"),
            "{malformed}"
        );
    }

    #[test]
    fn the_event_time_is_kept_as_offered() {
        let line = offer_line(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T16:02:11+02:00""#,
        );
        let Ok(Line::Offer(offer)) = read_line(&line) else {
            panic!("not an offer");
        };
        assert_eq!(offer.event_at, "2026-10-10T16:02:11+02:00");
    }

    #[test]
    fn a_bad_key_refuses_the_offer_naming_the_rule_and_the_key() {
        let malformed = refused(&offer_line(
            r#""key": "moves/x", "source": "release", "event_at": "2026-10-10T14:02:11Z""#,
        ));
        assert_eq!(malformed.field.as_deref(), Some("key"));
        assert_eq!(malformed.key.as_deref(), Some("moves/x"));
        assert!(malformed.reason.contains("`moves`"), "{malformed}");
    }

    #[test]
    fn an_unknown_field_of_an_offer_is_ignored() {
        let line = offer_line(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11Z", "weight": 3, "title": "x 1.0", "url": "https://example.org/x""#,
        );
        let Ok(Line::Offer(offer)) = read_line(&line) else {
            panic!("not an offer");
        };
        assert_eq!(offer.title.as_deref(), Some("x 1.0"));
        assert_eq!(offer.url.as_deref(), Some("https://example.org/x"));
    }

    fn excerpt_offer(excerpt: &str) -> String {
        offer_line(&format!(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11Z", "excerpt": {excerpt}"#
        ))
    }

    fn with_field(field: &str, value: &str) -> String {
        let mut excerpt: Map<String, Value> = serde_json::from_str(EXCERPT).unwrap();
        if value.is_empty() {
            excerpt.remove(field);
        } else {
            excerpt.insert(field.into(), serde_json::from_str(value).unwrap());
        }
        excerpt_offer(&Value::Object(excerpt).to_string())
    }

    #[test]
    fn an_excerpt_with_no_text_is_refused() {
        let malformed = refused(&with_field("text", r#""""#));
        assert_eq!(malformed.field.as_deref(), Some("excerpt.text"));
        assert_eq!(malformed.to_string(), "`excerpt.text` is empty");
    }

    #[test]
    fn an_excerpt_of_an_unknown_nature_is_refused() {
        let malformed = refused(&with_field("kind", r#""opinion""#));
        assert_eq!(malformed.field.as_deref(), Some("excerpt.kind"));
        assert!(malformed.reason.contains("`opinion`"), "{malformed}");
    }

    #[test]
    fn an_excerpt_lacking_any_field_is_refused_naming_it() {
        for field in [
            "text",
            "kind",
            "asserted_by",
            "assertion",
            "subjects",
            "argues_with",
            "position",
        ] {
            let malformed = refused(&with_field(field, ""));
            assert_eq!(
                malformed.field,
                Some(format!("excerpt.{field}")),
                "{malformed}"
            );
            assert_eq!(malformed.reason, "is missing");
        }
        let wrong = refused(&with_field("subjects", r#"["a", 2]"#));
        assert_eq!(wrong.field.as_deref(), Some("excerpt.subjects[1]"));
    }

    #[test]
    fn an_excerpt_with_empty_lists_is_taken() {
        let mut line = with_field("subjects", "[]");
        line = line.replace(r#"["packages/mise/default.nix"]"#, "[]");
        let Ok(Line::Offer(offer)) = read_line(&line) else {
            panic!("not an offer: {line}");
        };
        let excerpt = offer.excerpt.expect("an own excerpt");
        assert!(excerpt.subjects.is_empty());
        assert!(excerpt.argues_with.is_empty());
        assert_eq!(excerpt.kind, Nature::Ask);
        assert_eq!(excerpt.text, "mise 2026.10.10 is out.\nIt fixes the shim.");
        assert_eq!(excerpt.position, "release notes");
    }

    fn raw_offer(raw: &str) -> String {
        offer_line(&format!(
            r#""key": "release/x", "source": "release", "event_at": "2026-10-10T14:02:11Z", "raw": {raw}"#
        ))
    }

    #[test]
    fn a_raw_path_outside_the_raw_place_is_refused() {
        let malformed = refused(&raw_offer(r#"["notes.md", "../elsewhere.md"]"#));
        assert_eq!(malformed.field.as_deref(), Some("raw[1]"));
        assert!(
            malformed.reason.contains("`../elsewhere.md`"),
            "{malformed}"
        );

        let absolute = refused(&raw_offer(r#"["/etc/passwd"]"#));
        assert!(absolute.reason.contains("`/etc/passwd`"), "{absolute}");

        let deep = refused(&raw_offer(r#"["a/../../b"]"#));
        assert!(deep.reason.contains("`..`"), "{deep}");
    }

    #[test]
    fn raw_paths_inside_the_raw_place_are_read() {
        let Ok(Line::Offer(offer)) = read_line(&raw_offer(r#"["m/notes.md", "m/release.json"]"#))
        else {
            panic!("not an offer");
        };
        assert_eq!(offer.raw, ["m/notes.md", "m/release.json"]);
    }

    #[test]
    fn a_cursor_is_kept_byte_for_byte() {
        assert_eq!(
            read_line(r#"{"cursor": "page=3&since=2026-10-17"}"#),
            Ok(Line::Cursor("page=3&since=2026-10-17".into()))
        );
        assert_eq!(refused(r#"{"cursor": 3}"#).field.as_deref(), Some("cursor"));
    }

    #[test]
    fn a_completion_lacking_a_failures_why_is_malformed() {
        let malformed = refused(r#"{"done": {"listed": 2, "failed": [{"what": "x"}]}}"#);
        assert_eq!(malformed.field.as_deref(), Some("done.failed[0].why"));
        assert_eq!(malformed.to_string(), "`done.failed[0].why` is missing");
    }

    #[test]
    fn lines_are_written_as_the_host_reads_them() {
        let offer = Line::Offer(Offer {
            key: Key::parse("release/mise/2026.10.10").unwrap(),
            source: "release".into(),
            event_at: "2026-10-10T16:02:11+02:00".into(),
            title: Some("mise 2026.10.10".into()),
            url: Some("https://github.com/jdx/mise/releases/tag/v2026.10.10".into()),
            raw: vec!["mise-2026.10.10/notes.md".into()],
            excerpt: Some(Excerpt {
                text: "## Fixes\n\n- the shim\n- the \"quoted\" path\r\n".into(),
                kind: Nature::Ask,
                asserted_by: "jdx/mise".into(),
                assertion: "mise 2026.10.10 is released.".into(),
                subjects: vec!["release".into(), "mise".into()],
                argues_with: vec![],
                position: "release notes".into(),
            }),
        });
        let bare = Line::Offer(Offer {
            key: Key::parse("deploy/a/1").unwrap(),
            source: "deploy".into(),
            event_at: "2026-10-10T14:02:11Z".into(),
            title: None,
            url: None,
            raw: vec![],
            excerpt: None,
        });
        let cursor = Line::Cursor("page=3&since=2026-10-17\n".into());
        let done = Line::Done(Completion {
            listed: 14,
            failed: vec![Failure {
                what: "github:jdx/mise".into(),
                why: "HTTP 502".into(),
            }],
        });
        for line in [offer, bare, cursor, done] {
            let written = line.to_json();
            assert!(!written.contains('\n'), "{written}");
            assert_eq!(read_line(&written), Ok(line));
        }
        let written = Line::Cursor("c1".into()).to_json();
        assert_eq!(written, r#"{"cursor":"c1"}"#);
    }

    #[test]
    fn a_completion_holds_its_count_and_failures() {
        assert_eq!(
            read_line(
                r#"{"done": {"listed": 14, "failed": [{"what": "github:jdx/mise", "why": "HTTP 502"}]}}"#
            ),
            Ok(Line::Done(Completion {
                listed: 14,
                failed: vec![Failure {
                    what: "github:jdx/mise".into(),
                    why: "HTTP 502".into()
                }]
            }))
        );
        for (line, field) in [
            (r#"{"done": {"failed": []}}"#, "done.listed"),
            (r#"{"done": {"listed": -1, "failed": []}}"#, "done.listed"),
            (r#"{"done": {"listed": 1}}"#, "done.failed"),
            (r#"{"done": {"listed": 1, "failed": {}}}"#, "done.failed"),
        ] {
            assert_eq!(refused(line).field.as_deref(), Some(field), "{line}");
        }
    }
}
