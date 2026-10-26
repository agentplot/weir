//! The import line, the fourth line C.1 shows: printed only by an import
//! and read only by a host's import operation, never by `take` (63, 64,
//! 75a). It carries one capture read before its inbox existed, with the
//! pointer, signals and moves it was read with (37 to 39). It is read as an
//! adapter's lines are, each field checked by name so a refusal names it,
//! as `import.signals[1].kind`, and a field the line's shape does not name
//! ignored (core.6); it is written as one line of compact JSON, which reads
//! back the same (C.1).

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

use crate::capture::CameIn;
use crate::key::Key;
use crate::line::{
    read_event_at, read_key, read_nature, read_source, required, required_string, string, strings,
    Malformed, Nature,
};

/// One capture an import read, with its pointer, signals and moves (63).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImportLine {
    pub capture: ImportLineCapture,
    /// As the body gave it, byte for byte, and never checked (37).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    /// Always `import:<body>` (22).
    #[serde(serialize_with = "came_in")]
    pub came_in: CameIn,
    /// The body's, in its order, numbered from 1 (27).
    pub signals: Vec<ImportLineSignal>,
    pub moves: Vec<ImportLineMove>,
}

/// The capture: an offer's object without raw material or an excerpt
/// (37, 63; C.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImportLineCapture {
    pub key: Key,
    pub source: String,
    /// In RFC 3339 with its offset, as the body gave it (5; core.8).
    pub event_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// One signal as the body read it: every field of a signal, its excerpt
/// never cut or looked for (25, 37).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImportLineSignal {
    /// Its number in the body's order, from 1 (27).
    pub n: u64,
    pub kind: Nature,
    pub asserted_by: String,
    pub assertion: String,
    pub subjects: Vec<String>,
    pub argues_with: Vec<String>,
    /// Never empty.
    pub excerpt: String,
    pub position: String,
}

/// One move the body made on a signal, by the signal's number. Its word is
/// kept as it stands, one of the six or not, with its consequence or not,
/// for the import operation to take or leave unmoved (30, 31, 39; core.25).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImportLineMove {
    pub signal: u64,
    #[serde(rename = "move")]
    pub word: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

fn came_in<S: Serializer>(came_in: &CameIn, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(came_in)
}

const ADAPTER_KEYS: [&str; 3] = ["capture", "cursor", "done"];

/// The import line `text` is, or why it is malformed.
pub fn read_import_line(text: &str) -> Result<ImportLine, Malformed> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| Malformed::line(format!("the line is not JSON: {error}")))?;
    let Value::Object(object) = value else {
        return Err(Malformed::line("the line is not a JSON object"));
    };
    let held: Vec<String> = ADAPTER_KEYS
        .into_iter()
        .filter(|name| object.contains_key(*name))
        .map(|name| format!("`{name}`"))
        .collect();
    let Some(import) = object.get("import") else {
        return Err(Malformed::line("the line holds no `import`"));
    };
    if !held.is_empty() {
        return Err(Malformed::line(format!(
            "the line holds `import` and {}, and an import line holds none of `capture`, \
             `cursor` and `done`",
            held.join(" and ")
        )));
    }
    let Value::Object(import) = import else {
        return Err(Malformed::field("import", "is not an object"));
    };
    let key_text = import
        .get("capture")
        .and_then(|capture| capture.get("key"))
        .and_then(Value::as_str)
        .map(str::to_string);
    import_fields(import).map_err(|mut malformed| {
        malformed.key = key_text;
        malformed
    })
}

fn import_fields(import: &Map<String, Value>) -> Result<ImportLine, Malformed> {
    let capture = read_capture(required(import, "capture", "import.capture")?)?;
    let pointer = string(import, "pointer", "import.pointer")?;
    if pointer.as_deref() == Some("") {
        return Err(Malformed::field("import.pointer", "is empty"));
    }
    let came_in_text = required_string(import, "came_in", "import.came_in")?;
    let came_in = match CameIn::parse(&came_in_text) {
        Some(came_in @ CameIn::Import(_)) => came_in,
        _ => {
            return Err(Malformed::field(
                "import.came_in",
                format!("is `{came_in_text}`, not import:<body>"),
            ))
        }
    };
    let signals = list(import, "signals")?
        .iter()
        .enumerate()
        .map(|(n, entry)| read_signal(n, entry))
        .collect::<Result<_, _>>()?;
    let moves = list(import, "moves")?
        .iter()
        .enumerate()
        .map(|(n, entry)| read_move(n, entry))
        .collect::<Result<_, _>>()?;
    Ok(ImportLine {
        capture,
        pointer,
        came_in,
        signals,
        moves,
    })
}

fn read_capture(value: &Value) -> Result<ImportLineCapture, Malformed> {
    let Value::Object(capture) = value else {
        return Err(Malformed::field("import.capture", "is not an object"));
    };
    let key = read_key(capture, "import.capture.key")?;
    let source = read_source(capture, "import.capture.source")?;
    let event_at = read_event_at(capture, "import.capture.event_at")?;
    let title = string(capture, "title", "import.capture.title")?;
    let url = string(capture, "url", "import.capture.url")?;
    for (name, why) in [
        ("raw", "an import copies no raw material"),
        ("excerpt", "an imported capture's signals are its reading"),
    ] {
        if capture.contains_key(name) {
            return Err(Malformed::field(
                format!("import.capture.{name}"),
                format!("is given, and {why}"),
            ));
        }
    }
    Ok(ImportLineCapture {
        key,
        source,
        event_at,
        title,
        url,
    })
}

fn list<'a>(import: &'a Map<String, Value>, name: &str) -> Result<&'a Vec<Value>, Malformed> {
    let path = format!("import.{name}");
    match required(import, name, &path)? {
        Value::Array(entries) => Ok(entries),
        _ => Err(Malformed::field(path, "is not a list")),
    }
}

fn whole_number(object: &Map<String, Value>, name: &str, path: &str) -> Result<u64, Malformed> {
    required(object, name, path)?
        .as_u64()
        .ok_or_else(|| Malformed::field(path, "is not a whole number"))
}

fn read_signal(index: usize, value: &Value) -> Result<ImportLineSignal, Malformed> {
    let path = format!("import.signals[{index}]");
    let Value::Object(signal) = value else {
        return Err(Malformed::field(path, "is not an object"));
    };
    let at = |name: &str| format!("{path}.{name}");
    let n = whole_number(signal, "n", &at("n"))?;
    let expected = index as u64 + 1;
    if n != expected {
        return Err(Malformed::field(
            at("n"),
            format!("is {n}, and signal {expected} of the list is numbered {expected}"),
        ));
    }
    let kind = read_nature(signal, "kind", &at("kind"))?;
    let asserted_by = required_string(signal, "asserted_by", &at("asserted_by"))?;
    let assertion = required_string(signal, "assertion", &at("assertion"))?;
    let subjects = required(signal, "subjects", &at("subjects"))
        .and_then(|value| strings(value, &at("subjects")))?;
    let argues_with = required(signal, "argues_with", &at("argues_with"))
        .and_then(|value| strings(value, &at("argues_with")))?;
    let excerpt = required_string(signal, "excerpt", &at("excerpt"))?;
    if excerpt.is_empty() {
        return Err(Malformed::field(at("excerpt"), "is empty"));
    }
    let position = required_string(signal, "position", &at("position"))?;
    Ok(ImportLineSignal {
        n,
        kind,
        asserted_by,
        assertion,
        subjects,
        argues_with,
        excerpt,
        position,
    })
}

fn read_move(index: usize, value: &Value) -> Result<ImportLineMove, Malformed> {
    let path = format!("import.moves[{index}]");
    let Value::Object(entry) = value else {
        return Err(Malformed::field(path, "is not an object"));
    };
    let at = |name: &str| format!("{path}.{name}");
    let signal = whole_number(entry, "signal", &at("signal"))?;
    let word = required_string(entry, "move", &at("move"))?;
    if word.is_empty() {
        return Err(Malformed::field(at("move"), "is empty"));
    }
    Ok(ImportLineMove {
        signal,
        word,
        target: string(entry, "target", &at("target"))?,
        by: string(entry, "by", &at("by"))?,
        at: string(entry, "at", &at("at"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = r#"{"import": {"capture": {"key": "release/mise/2026.10.10", "source": "release", "event_at": "2026-10-10T14:02:11Z", "title": "mise 2026.10.10"}, "pointer": "mac-studio:~/.local/state/x/", "came_in": "import:crew-folders", "signals": [{"n": 1, "kind": "ask", "asserted_by": "jdx/mise", "assertion": "mise 2026.10.10 is released.", "subjects": ["release"], "argues_with": [], "excerpt": "notes", "position": "release notes"}], "moves": [{"signal": 1, "move": "route", "target": "unit/bump-mise", "by": "planner", "at": "2026-10-11T00:00:00Z"}]}}"#;

    fn refused(line: &str) -> Malformed {
        read_import_line(line).expect_err(line)
    }

    /// The release line with `edit` applied to its `import` object.
    fn edited(edit: impl FnOnce(&mut Map<String, Value>)) -> String {
        let mut line: Map<String, Value> = serde_json::from_str(RELEASE).unwrap();
        let Some(Value::Object(import)) = line.get_mut("import") else {
            unreachable!()
        };
        edit(import);
        Value::Object(line).to_string()
    }

    fn object(value: &mut Value) -> &mut Map<String, Value> {
        value.as_object_mut().expect("an object")
    }

    #[test]
    fn a_release_capture_with_one_signal_and_one_move() {
        let line = read_import_line(RELEASE).unwrap();
        assert_eq!(
            line,
            ImportLine {
                capture: ImportLineCapture {
                    key: Key::parse("release/mise/2026.10.10").unwrap(),
                    source: "release".into(),
                    event_at: "2026-10-10T14:02:11Z".into(),
                    title: Some("mise 2026.10.10".into()),
                    url: None,
                },
                pointer: Some("mac-studio:~/.local/state/x/".into()),
                came_in: CameIn::Import("crew-folders".into()),
                signals: vec![ImportLineSignal {
                    n: 1,
                    kind: Nature::Ask,
                    asserted_by: "jdx/mise".into(),
                    assertion: "mise 2026.10.10 is released.".into(),
                    subjects: vec!["release".into()],
                    argues_with: vec![],
                    excerpt: "notes".into(),
                    position: "release notes".into(),
                }],
                moves: vec![ImportLineMove {
                    signal: 1,
                    word: "route".into(),
                    target: Some("unit/bump-mise".into()),
                    by: Some("planner".into()),
                    at: Some("2026-10-11T00:00:00Z".into()),
                }],
            }
        );
    }

    #[test]
    fn a_line_holding_an_adapters_key_as_well() {
        let line = RELEASE.replacen("{\"import\"", "{\"cursor\": \"c1\", \"import\"", 1);
        let malformed = refused(&line);
        assert_eq!(malformed.field, None);
        assert!(
            malformed.reason.contains("`import` and `cursor`"),
            "{malformed}"
        );

        let neither = refused(r#"{"cursor": "c1"}"#);
        assert_eq!(neither.reason, "the line holds no `import`");
        assert!(refused("not json")
            .reason
            .starts_with("the line is not JSON"));
        assert_eq!(
            refused(r#"{"import": []}"#).field.as_deref(),
            Some("import")
        );
    }

    #[test]
    fn an_unknown_field_is_ignored() {
        let line = edited(|import| {
            import.insert("trace".into(), Value::from(3));
            object(&mut import["capture"]).insert("weight".into(), Value::from(1));
            object(&mut import["signals"][0]).insert("grade".into(), "verified".into());
            object(&mut import["moves"][0]).insert("reason".into(), "why".into());
        });
        assert_eq!(read_import_line(&line), read_import_line(RELEASE));
        let outer = RELEASE.replacen("{\"import\"", "{\"note\": 1, \"import\"", 1);
        assert_eq!(read_import_line(&outer), read_import_line(RELEASE));
    }

    #[test]
    fn a_key_the_rule_refuses() {
        let line = edited(|import| {
            import["capture"]["key"] = "moves/x".into();
        });
        let malformed = refused(&line);
        assert_eq!(malformed.field.as_deref(), Some("import.capture.key"));
        assert!(malformed.reason.contains("`moves`"), "{malformed}");
        assert_eq!(malformed.key.as_deref(), Some("moves/x"));
    }

    #[test]
    fn a_refusal_names_the_key_where_one_was_read() {
        let line = edited(|import| {
            import["signals"][0]["kind"] = "finding".into();
        });
        assert_eq!(
            refused(&line).key.as_deref(),
            Some("release/mise/2026.10.10")
        );
        let no_key = edited(|import| {
            object(&mut import["capture"]).remove("key");
        });
        let malformed = refused(&no_key);
        assert_eq!(malformed.field.as_deref(), Some("import.capture.key"));
        assert_eq!(malformed.key, None);
    }

    #[test]
    fn the_capture_needs_its_source_and_event_time() {
        for (name, value, reason) in [
            ("source", None, "is missing"),
            ("source", Some(""), "is empty"),
            ("event_at", None, "is missing"),
            ("event_at", Some("2026-10-10T14:02:11"), "not an RFC 3339"),
        ] {
            let line = edited(|import| {
                let capture = object(&mut import["capture"]);
                match value {
                    None => drop(capture.remove(name)),
                    Some(value) => drop(capture.insert(name.into(), value.into())),
                }
            });
            let malformed = refused(&line);
            assert_eq!(
                malformed.field,
                Some(format!("import.capture.{name}")),
                "{line}"
            );
            assert!(malformed.reason.contains(reason), "{malformed}");
        }
        let offset = edited(|import| {
            import["capture"]["event_at"] = "2026-10-10T16:02:11+02:00".into();
        });
        assert_eq!(
            read_import_line(&offset).unwrap().capture.event_at,
            "2026-10-10T16:02:11+02:00"
        );
    }

    #[test]
    fn raw_material_named_in_an_import() {
        for (name, value) in [
            ("raw", serde_json::json!(["notes.md"])),
            ("excerpt", serde_json::json!({"text": "x"})),
        ] {
            let line = edited(|import| {
                object(&mut import["capture"]).insert(name.into(), value);
            });
            let malformed = refused(&line);
            assert_eq!(
                malformed.field,
                Some(format!("import.capture.{name}")),
                "{malformed}"
            );
        }
    }

    #[test]
    fn a_pointer_of_another_machine() {
        let line = edited(|import| {
            import["pointer"] = "deneb:~/.local/state/crew/raw/a.jsonl".into();
        });
        assert_eq!(
            read_import_line(&line).unwrap().pointer.as_deref(),
            Some("deneb:~/.local/state/crew/raw/a.jsonl")
        );
        let none = edited(|import| {
            import.remove("pointer");
        });
        assert_eq!(read_import_line(&none).unwrap().pointer, None);
        let empty = edited(|import| {
            import["pointer"] = "".into();
        });
        assert_eq!(refused(&empty).field.as_deref(), Some("import.pointer"));
    }

    #[test]
    fn a_came_in_of_an_adapters_run() {
        for came_in in ["run:folder", "intake", "import:", "cron"] {
            let line = edited(|import| {
                import["came_in"] = came_in.into();
            });
            let malformed = refused(&line);
            assert_eq!(
                malformed.field.as_deref(),
                Some("import.came_in"),
                "{came_in}"
            );
            assert!(malformed.reason.contains(came_in), "{malformed}");
        }
        let missing = edited(|import| {
            import.remove("came_in");
        });
        assert_eq!(refused(&missing).field.as_deref(), Some("import.came_in"));
    }

    #[test]
    fn a_capture_with_no_signals() {
        let line = edited(|import| {
            import["signals"] = serde_json::json!([]);
            import["moves"] = serde_json::json!([]);
        });
        let read = read_import_line(&line).unwrap();
        assert!(read.signals.is_empty());
        assert!(read.moves.is_empty());
        for name in ["signals", "moves"] {
            let missing = edited(|import| {
                import.remove(name);
            });
            assert_eq!(refused(&missing).field, Some(format!("import.{name}")));
            let not_list = edited(|import| {
                import[name] = serde_json::json!({});
            });
            assert_eq!(refused(&not_list).field, Some(format!("import.{name}")));
        }
    }

    fn two_signals(edit: impl FnOnce(&mut Map<String, Value>)) -> String {
        edited(|import| {
            let mut second = import["signals"][0].clone();
            second["n"] = 2.into();
            edit(object(&mut second));
            import["signals"].as_array_mut().unwrap().push(second);
        })
    }

    #[test]
    fn a_nature_not_of_the_five() {
        let line = two_signals(|signal| {
            signal.insert("kind".into(), "finding".into());
        });
        let malformed = refused(&line);
        assert_eq!(malformed.field.as_deref(), Some("import.signals[1].kind"));
        assert!(malformed.reason.contains("`finding`"), "{malformed}");
        assert_eq!(
            read_import_line(&two_signals(|_| {}))
                .unwrap()
                .signals
                .len(),
            2
        );
    }

    #[test]
    fn a_signal_lacking_any_field_refuses_the_line_naming_it() {
        for name in [
            "n",
            "kind",
            "asserted_by",
            "assertion",
            "subjects",
            "argues_with",
            "excerpt",
            "position",
        ] {
            let line = two_signals(|signal| {
                signal.remove(name);
            });
            let malformed = refused(&line);
            assert_eq!(
                malformed.field,
                Some(format!("import.signals[1].{name}")),
                "{malformed}"
            );
            assert_eq!(malformed.reason, "is missing");
        }
        let wrong = two_signals(|signal| {
            signal.insert("subjects".into(), serde_json::json!(["a", 2]));
        });
        assert_eq!(
            refused(&wrong).field.as_deref(),
            Some("import.signals[1].subjects[1]")
        );
        let empty = two_signals(|signal| {
            signal.insert("excerpt".into(), "".into());
        });
        let malformed = refused(&empty);
        assert_eq!(
            malformed.field.as_deref(),
            Some("import.signals[1].excerpt")
        );
        assert_eq!(malformed.reason, "is empty");
        let not_object = edited(|import| {
            import["signals"] = serde_json::json!(["a"]);
        });
        assert_eq!(
            refused(&not_object).field.as_deref(),
            Some("import.signals[0]")
        );
    }

    #[test]
    fn signals_numbered_out_of_order() {
        let line = two_signals(|signal| {
            signal.insert("n".into(), 3.into());
        });
        let malformed = refused(&line);
        assert_eq!(malformed.field.as_deref(), Some("import.signals[1].n"));
        assert!(malformed.reason.contains("is 3"), "{malformed}");
        let from_zero = edited(|import| {
            import["signals"][0]["n"] = 0.into();
        });
        assert_eq!(
            refused(&from_zero).field.as_deref(),
            Some("import.signals[0].n")
        );
        let not_whole = two_signals(|signal| {
            signal.insert("n".into(), 2.5.into());
        });
        assert_eq!(refused(&not_whole).reason, "is not a whole number");
    }

    fn with_move(entry: Value) -> String {
        edited(|import| {
            import["moves"] = Value::Array(vec![entry]);
        })
    }

    #[test]
    fn a_word_not_of_the_six() {
        let line = with_move(serde_json::json!(
            {"signal": 1, "move": "promote", "by": "curation", "at": "2026-10-12T00:00:00Z"}
        ));
        assert_eq!(
            read_import_line(&line).unwrap().moves,
            [ImportLineMove {
                signal: 1,
                word: "promote".into(),
                target: None,
                by: Some("curation".into()),
                at: Some("2026-10-12T00:00:00Z".into()),
            }]
        );
    }

    #[test]
    fn a_drop_with_no_reason() {
        let line = with_move(serde_json::json!({"signal": 1, "move": "drop"}));
        assert_eq!(
            read_import_line(&line).unwrap().moves,
            [ImportLineMove {
                signal: 1,
                word: "drop".into(),
                target: None,
                by: None,
                at: None,
            }]
        );
    }

    #[test]
    fn a_move_of_a_signal_no_capture_holds_is_read() {
        let line = with_move(serde_json::json!({"signal": 7, "move": "attach", "target": "x"}));
        assert_eq!(read_import_line(&line).unwrap().moves[0].signal, 7);
    }

    #[test]
    fn a_move_refuses_the_line_only_for_its_number_or_word() {
        for (entry, field) in [
            (
                serde_json::json!({"move": "drop"}),
                "import.moves[0].signal",
            ),
            (
                serde_json::json!({"signal": "1", "move": "drop"}),
                "import.moves[0].signal",
            ),
            (
                serde_json::json!({"signal": -1, "move": "drop"}),
                "import.moves[0].signal",
            ),
            (serde_json::json!({"signal": 1}), "import.moves[0].move"),
            (
                serde_json::json!({"signal": 1, "move": ""}),
                "import.moves[0].move",
            ),
            (
                serde_json::json!({"signal": 1, "move": 3}),
                "import.moves[0].move",
            ),
            (
                serde_json::json!({"signal": 1, "move": "drop", "by": 3}),
                "import.moves[0].by",
            ),
            (serde_json::json!("drop"), "import.moves[0]"),
        ] {
            let line = with_move(entry);
            assert_eq!(refused(&line).field.as_deref(), Some(field), "{line}");
        }
    }
}
