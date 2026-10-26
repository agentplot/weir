//! The readings every table of the format shares: a field's path, a flag,
//! a list of strings, and the keys the format does not name.

use toml::{Table, Value};

use crate::refusal::Problems;

/// A key as a field's path writes it: bare where TOML allows a bare key,
/// quoted otherwise, so that `kinds."a.b"` is never read as `kinds.a.b`.
pub(crate) fn key(name: &str) -> String {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare {
        name.to_owned()
    } else {
        Value::String(name.to_owned()).to_string()
    }
}

/// The path of the field `name` of the table at `at`.
pub(crate) fn field(at: &str, name: &str) -> String {
    format!("{at}.{}", key(name))
}

/// A flag: false when left out, refused when not a boolean.
pub(crate) fn boolean(table: &Table, name: &str, at: &str, problems: &mut Problems) -> bool {
    match table.get(name) {
        None => false,
        Some(Value::Boolean(flag)) => *flag,
        Some(_) => {
            problems.refuse(&field(at, name), "must be true or false");
            false
        }
    }
}

/// A list of strings: empty when left out, refused when not an array of
/// strings.
pub(crate) fn strings(table: &Table, name: &str, at: &str, problems: &mut Problems) -> Vec<String> {
    let Some(value) = table.get(name) else {
        return Vec::new();
    };
    match value.as_array().and_then(|items| {
        items
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
    }) {
        Some(items) => items,
        None => {
            problems.refuse(&field(at, name), "must be an array of strings");
            Vec::new()
        }
    }
}

/// Every key of `table` but those the format names, unread and as written
/// (56).
pub(crate) fn unnamed(table: &Table, named: &[&str]) -> Table {
    table
        .iter()
        .filter(|(name, _)| !named.contains(&name.as_str()))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}
