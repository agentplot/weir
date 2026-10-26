//! The key rule: a key is made of `/`-separated segments and names nothing
//! in an inbox but its own capture (3, 4; C.1; core.7).

use std::fmt;

use serde::{Serialize, Serializer};

/// The first segments a key never has: where captures, moves and items lie
/// in an inbox (C.2; record 0002).
const ROOTS: [&str; 3] = ["captures", "moves", "items"];

/// The files the layout places, each with what it holds (C.2). Each segment
/// of a key is a folder, so a segment of one of these names would stand
/// where that file lies: `x/capture.rec` where capture `x`'s record does.
/// A later record's file is added here.
const LAYOUT_FILES: [(&str, &str); 6] = [
    ("capture.rec", "a capture's record"),
    ("signals.rec", "a capture's signals"),
    ("moves.rec", "the inbox's moves"),
    ("item.rec", "an item's record"),
    ("brief.md", "an item's brief"),
    ("sends.rec", "an item's sends"),
];

/// A key that keeps the rule. Only [`Key::parse`] makes one.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(String);

/// Why a key was refused: the rule it breaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyError {
    Empty,
    LeadingSlash,
    /// `a//b` or a trailing `/`: the same place as a key without it (42).
    EmptySegment,
    /// `.` or `..`: the same place as another key, or a place outside.
    DotSegment(String),
    /// A segment that is the name of a file the layout places.
    LayoutFile(String),
    /// `captures`, `moves` or `items` as the first segment.
    Root(String),
}

impl Key {
    /// The key `text` names, or the rule it breaks.
    pub fn parse(text: &str) -> Result<Key, KeyError> {
        if text.is_empty() {
            return Err(KeyError::Empty);
        }
        if text.starts_with('/') {
            return Err(KeyError::LeadingSlash);
        }
        for segment in text.split('/') {
            match segment {
                "" => return Err(KeyError::EmptySegment),
                "." | ".." => return Err(KeyError::DotSegment(segment.to_string())),
                _ if LAYOUT_FILES.iter().any(|(file, _)| *file == segment) => {
                    return Err(KeyError::LayoutFile(segment.to_string()))
                }
                _ => {}
            }
        }
        let first = text.split('/').next().unwrap_or_default();
        if ROOTS.contains(&first) {
            return Err(KeyError::Root(first.to_string()));
        }
        Ok(Key(text.to_string()))
    }

    /// The key's segments, in order.
    pub fn segments(&self) -> Vec<&str> {
        self.0.split('/').collect()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::Empty => write!(f, "a key is never empty"),
            KeyError::LeadingSlash => write!(f, "a key never starts with `/`"),
            KeyError::EmptySegment => write!(f, "a key never has an empty segment"),
            KeyError::DotSegment(segment) => write!(f, "a key never has a `{segment}` segment"),
            KeyError::LayoutFile(segment) => {
                let holds = LAYOUT_FILES
                    .iter()
                    .find(|(file, _)| file == segment)
                    .map_or("a file of the layout", |(_, holds)| holds);
                write!(
                    f,
                    "a key never has a `{segment}` segment, which would stand where the file `{segment}`, {holds}, lies"
                )
            }
            KeyError::Root(root) => write!(
                f,
                "a key's first segment is never `{root}`, where an inbox's {root} lie"
            ),
        }
    }
}

impl std::error::Error for KeyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_that_reaches_another_root_is_refused() {
        for (text, refusal) in [
            ("moves/x", KeyError::Root("moves".into())),
            ("captures/x", KeyError::Root("captures".into())),
            ("items/x", KeyError::Root("items".into())),
            ("/release/x", KeyError::LeadingSlash),
            ("release/../x", KeyError::DotSegment("..".into())),
        ] {
            assert_eq!(Key::parse(text), Err(refusal), "{text}");
        }
        assert_eq!(
            Key::parse("moves/x").unwrap_err().to_string(),
            "a key's first segment is never `moves`, where an inbox's moves lie"
        );
        assert_eq!(
            Key::parse("release/../x").unwrap_err().to_string(),
            "a key never has a `..` segment"
        );
    }

    #[test]
    fn a_key_that_names_another_keys_place_is_refused() {
        for (text, refusal) in [
            ("release//x", KeyError::EmptySegment),
            ("release/x/", KeyError::EmptySegment),
            ("release/./x", KeyError::DotSegment(".".into())),
            ("", KeyError::Empty),
        ] {
            assert_eq!(Key::parse(text), Err(refusal), "{text:?}");
        }
        assert_eq!(
            Key::parse("").unwrap_err().to_string(),
            "a key is never empty"
        );
    }

    #[test]
    fn a_key_that_names_a_file_of_the_layout_is_refused() {
        for (text, file) in [
            ("release/capture.rec", "capture.rec"),
            ("release/x/signals.rec", "signals.rec"),
            ("capture.rec", "capture.rec"),
            ("deploy/moves.rec", "moves.rec"),
            ("deploy/item.rec", "item.rec"),
            ("deploy/brief.md", "brief.md"),
            ("deploy/sends.rec", "sends.rec"),
        ] {
            assert_eq!(
                Key::parse(text),
                Err(KeyError::LayoutFile(file.into())),
                "{text}"
            );
        }
        assert_eq!(
            Key::parse("release/x/signals.rec").unwrap_err().to_string(),
            "a key never has a `signals.rec` segment, which would stand where the file \
             `signals.rec`, a capture's signals, lies"
        );
        assert_eq!(
            Key::parse("deploy/brief.md").unwrap_err().to_string(),
            "a key never has a `brief.md` segment, which would stand where the file \
             `brief.md`, an item's brief, lies"
        );
        // Every name the layout places is one the rule refuses.
        for (file, _) in LAYOUT_FILES {
            assert!(Key::parse(&format!("x/{file}/y")).is_err(), "{file}");
        }
    }

    #[test]
    fn a_name_like_a_layout_file_is_taken() {
        for text in ["release/capture.rec.1", "deploy/brief", "notes/items.rec"] {
            assert!(Key::parse(text).is_ok(), "{text}");
        }
    }

    #[test]
    fn a_key_of_plain_segments_is_taken() {
        let key = Key::parse("release/mise/2026.10.10").unwrap();
        assert_eq!(key.segments(), ["release", "mise", "2026.10.10"]);
        assert_eq!(key.as_str(), "release/mise/2026.10.10");
    }

    #[test]
    fn a_root_name_past_the_first_segment_is_taken() {
        let key = Key::parse("deploy/moves/1").unwrap();
        assert_eq!(key.segments(), ["deploy", "moves", "1"]);
    }
}
