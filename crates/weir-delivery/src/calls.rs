//! The calls a deliverer makes over its own records (B.7), and the refusal
//! each gives naming the field that failed.

use std::fmt;

/// A call, or a read of a record, refused: the field that failed and why
/// (100, 108 to 110; core.47).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// The field refused, as the call's argument or the record's field is
    /// named: `key`, `readers`, `at`, `span` and the like.
    pub field: String,
    /// Why, in words a reader of the refusal is shown.
    pub reason: String,
}

impl Refusal {
    /// The refusal of `field`, for `reason`.
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Refusal {
        Refusal {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`: {}", self.field, self.reason)
    }
}

impl std::error::Error for Refusal {}
