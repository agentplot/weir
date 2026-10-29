use std::fmt;

/// How a stage repeats: the one way the format names (53, C.4). Its
/// [`Display`](fmt::Display) is its spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    /// Worked one group of its change's tasks at a time, spelled
    /// `per-task-group`; only on a stage whose deliverable is tasks ticked
    /// (57). How a reader works a repetition is its own (58).
    PerTaskGroup,
}

impl Repeat {
    /// The way a spelling names, if it names one.
    pub(crate) fn from_spelling(spelling: &str) -> Option<Self> {
        match spelling {
            "per-task-group" => Some(Self::PerTaskGroup),
            _ => None,
        }
    }
}

impl fmt::Display for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PerTaskGroup => f.write_str("per-task-group"),
        }
    }
}
