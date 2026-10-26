use std::fmt;

/// What a stage that is not a conversation leaves behind: one of six
/// forms, each spelled one way (54, C.4). Its [`Display`](fmt::Display)
/// is its spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deliverable {
    /// Commits, spelled `commits`.
    Commits,
    /// A change written, spelled `change`.
    Change,
    /// Tasks ticked, spelled `tasks`.
    Tasks,
    /// A change archived, spelled `archived`.
    Archived,
    /// A report at a path, spelled `report:<path>`.
    Report(String),
    /// A committed report at a path, spelled `committed:<path>`.
    Committed(String),
}

impl Deliverable {
    /// The form a spelling names, or why it names none.
    pub(crate) fn from_spelling(spelling: &str) -> Result<Self, String> {
        let at_path = |prefix: &str, path: &str, form: fn(String) -> Self| {
            if path.is_empty() {
                Err(format!("`{prefix}:` names no path"))
            } else {
                Ok(form(path.to_owned()))
            }
        };
        match spelling {
            "commits" => Ok(Self::Commits),
            "change" => Ok(Self::Change),
            "tasks" => Ok(Self::Tasks),
            "archived" => Ok(Self::Archived),
            _ => {
                if let Some(path) = spelling.strip_prefix("report:") {
                    at_path("report", path, Self::Report)
                } else if let Some(path) = spelling.strip_prefix("committed:") {
                    at_path("committed", path, Self::Committed)
                } else {
                    Err(format!(
                        "`{spelling}` is not one of commits, change, tasks, archived, \
                         report:<path> and committed:<path>"
                    ))
                }
            }
        }
    }
}

impl fmt::Display for Deliverable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Commits => f.write_str("commits"),
            Self::Change => f.write_str("change"),
            Self::Tasks => f.write_str("tasks"),
            Self::Archived => f.write_str("archived"),
            Self::Report(path) => write!(f, "report:{path}"),
            Self::Committed(path) => write!(f, "committed:{path}"),
        }
    }
}

/// How a stage ends: on its one deliverable, or as a conversation, which
/// has none (51, 53).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum End {
    /// The stage's one deliverable, from `deliverables` (54).
    Deliverable(Deliverable),
    /// The stage is a conversation: `conversation = true` (51).
    Conversation,
}
