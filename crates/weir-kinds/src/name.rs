//! A name, a source's or a kind's: one or more ASCII letters, digits, `_`
//! and `-` (C.4 as amended at blueprints 361a599; record 0004).

use std::fmt;

/// What a name is, as every reason that refuses one says it.
pub(crate) const NAME: &str = "one or more ASCII letters, digits, `_` and `-`";

/// Text refused as a name, an identity or a kind's name, with why (88,
/// C.4). Its [`Display`](fmt::Display) gives the text and the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameRefusal {
    /// The text refused, as given.
    pub text: String,
    /// Why it is refused, naming the part that fails and what that part
    /// must be.
    pub reason: String,
}

impl fmt::Display for NameRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text.is_empty() {
            write!(f, "the text {}", self.reason)
        } else {
            write!(f, "`{}` {}", self.text, self.reason)
        }
    }
}

impl std::error::Error for NameRefusal {}

/// Whether `text` is a name, a source's or a kind's: one or more ASCII
/// letters, digits, `_` and `-`, or refused saying why it is none (C.4 as
/// amended at blueprints 361a599; core.42; record 0004). Every read of a
/// name in this crate makes this one check.
pub fn check_name(text: &str) -> Result<(), NameRefusal> {
    match fault(text) {
        None => Ok(()),
        Some(fault) => Err(NameRefusal {
            text: text.to_owned(),
            reason: format!("{fault}, and a name is {NAME}"),
        }),
    }
}

/// Why `text` is not a name, or `None` when it is one.
pub(crate) fn fault(text: &str) -> Option<String> {
    if text.is_empty() {
        return Some("is empty".to_owned());
    }
    text.chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '_' || *c == '-'))
        .map(|c| format!("holds {c:?}"))
}

/// The reason `kinds.<key>` is refused when its key is not a name (57, 88).
pub(crate) fn not_a_kinds_name(key: &str) -> Option<String> {
    fault(key).map(|fault| format!("{fault}, and a kind's name is {NAME}"))
}
