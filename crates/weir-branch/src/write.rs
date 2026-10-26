//! The protocol: one write asked for, formed by its caller against the tip
//! read, checked there, made one commit, and landed only while the branch
//! still stands at that tip; a moved branch has it formed again on the new
//! tip, every check made again there, up to the writer's bound (44 to 50;
//! C.3 steps 1 to 5; core.26 to core.29).

use std::fmt;
use std::time::Duration;

use time::OffsetDateTime;

use crate::commit::{self, NotMade};
use crate::keeper::{self, check_path, Branch, Failure, Files, Moved, Step, Tip};
use crate::recfix;

/// How many times a write is formed again on a moved branch when its writer
/// sets no bound: tried at most once more than that (44; record 0002,
/// ruling 4).
pub const BOUND: u32 = 5;

/// A write asked for: what it does and who asked, each one line (50), how
/// many times it may be formed again on a moved branch, `None` for
/// [`BOUND`], the age past which a lock is stale (49), and the clock the
/// lock's times and the commit's date are read from (core.1).
pub struct Ask<'a> {
    pub what: &'a str,
    pub who: &'a str,
    pub bound: Option<u32>,
    pub stale_after: Duration,
    pub clock: &'a dyn Fn() -> OffsetDateTime,
}

/// A write that ended well: landed, or with nothing to write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written {
    Landed(Landed),
    Nothing(Nothing),
}

/// A write landed: its commit, the tip it landed on (none for the branch's
/// first), and how many times it was tried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub commit: String,
    pub on: Option<String>,
    pub tries: u32,
}

/// A write whose caller found nothing to write at the tip `at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nothing {
    pub at: Option<String>,
    pub tries: u32,
}

/// A write that did not land, the branch unmoved by it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// The ask names no one or nothing, or a branch or address git cannot
    /// be handed; refused before anything was read (50, 71).
    Asked { field: &'static str, reason: String },
    /// The caller's check refused the write at the tip `at` (46).
    Refused {
        at: Option<String>,
        reason: String,
        tries: u32,
    },
    /// A path of the write is not one a write may add or replace.
    Path {
        at: Option<String>,
        path: String,
        reason: String,
        tries: u32,
    },
    /// `recfix` refused a record file of the write (46; core.24).
    Recfix {
        at: Option<String>,
        path: String,
        said: String,
        tries: u32,
    },
    /// The branch moved on every try, past the bound (44).
    KeptMoving { tries: u32, bound: u32 },
    /// A step failed: git, `recfix` or the lock, with its words.
    Failed { step: Step, said: String },
}

impl WriteError {
    /// How many times the write was tried before it was refused.
    pub fn tries(&self) -> Option<u32> {
        match self {
            WriteError::Refused { tries, .. }
            | WriteError::Path { tries, .. }
            | WriteError::Recfix { tries, .. }
            | WriteError::KeptMoving { tries, .. } => Some(*tries),
            WriteError::Asked { .. } | WriteError::Failed { .. } => None,
        }
    }
}

fn tip_name(at: &Option<String>) -> String {
    match at {
        Some(commit) => format!("the tip {commit}"),
        None => "no tip".to_string(),
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Asked { field, reason } => {
                write!(
                    f,
                    "the write was refused before anything ran: `{field}`: {reason}"
                )
            }
            WriteError::Refused { at, reason, tries } => write!(
                f,
                "the write was refused at {} on try {tries}: {reason}",
                tip_name(at)
            ),
            WriteError::Path {
                at,
                path,
                reason,
                tries,
            } => write!(
                f,
                "the write was refused at {} on try {tries}: the path `{path}` {reason}",
                tip_name(at)
            ),
            WriteError::Recfix {
                at,
                path,
                said,
                tries,
            } => write!(
                f,
                "the write was refused at {} on try {tries}: recfix refused `{path}`: {said}",
                tip_name(at)
            ),
            WriteError::KeptMoving { tries, bound } => write!(
                f,
                "the write was refused: the branch kept moving, and it was tried {tries} times, \
                 its bound {bound}"
            ),
            WriteError::Failed { step, said } => write!(f, "the {step} failed: {said}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// One line, not empty: what a write does and who asked (50; core.29).
fn one_line(field: &'static str, value: &str) -> Result<(), WriteError> {
    let asked = |reason: &str| {
        Err(WriteError::Asked {
            field,
            reason: reason.to_string(),
        })
    };
    if value.trim().is_empty() {
        return asked("is empty");
    }
    if value.contains(['\n', '\r']) {
        return asked("holds more than one line");
    }
    Ok(())
}

/// The ask checked before anything is read or run (50, 71; core.3).
fn check_ask(branch: &Branch, ask: &Ask<'_>) -> Result<(), WriteError> {
    one_line("what", ask.what)?;
    one_line("who", ask.who)?;
    branch
        .check()
        .map_err(|(field, reason)| WriteError::Asked { field, reason })
}

/// Writes `branch` as `ask` says, the write formed by `form` from the tip
/// read: the files it adds or replaces, each with its full new text, none
/// for nothing to write, or `Err` naming why the caller refuses it there.
/// On a moved branch `form` is called again with the new tip.
pub fn write<F>(branch: &Branch, ask: &Ask<'_>, form: F) -> Result<Written, WriteError>
where
    F: FnMut(Option<&Tip>) -> Result<Files, String>,
{
    check_ask(branch, ask)?;
    if let keeper::Place::Remote { cache, .. } = branch.place() {
        std::fs::create_dir_all(cache).map_err(|error| WriteError::Failed {
            step: Step::Read,
            said: format!("the cache {} could not be made: {error}", cache.display()),
        })?;
    }
    tries(branch, ask, form)
}

/// Every try of the write, from the first read to its end.
fn tries<F>(branch: &Branch, ask: &Ask<'_>, mut form: F) -> Result<Written, WriteError>
where
    F: FnMut(Option<&Tip>) -> Result<Files, String>,
{
    let bound = ask.bound.unwrap_or(BOUND);
    let failed = |failure: Failure| WriteError::Failed {
        step: failure.step,
        said: failure.said,
    };
    let mut tip = keeper::read_tip(branch).map_err(failed)?;
    let mut tries = 0;
    loop {
        tries += 1;
        let at = tip.as_ref().map(|tip| tip.commit().to_string());
        let files = form(tip.as_ref()).map_err(|reason| WriteError::Refused {
            at: at.clone(),
            reason,
            tries,
        })?;
        if files.is_empty() {
            return Ok(Written::Nothing(Nothing { at, tries }));
        }
        for path in files.keys() {
            check_path(path).map_err(|reason| WriteError::Path {
                at: at.clone(),
                path: path.clone(),
                reason,
                tries,
            })?;
        }
        for (path, text) in files.iter().filter(|(path, _)| recfix::is_record(path)) {
            if let Some(said) = recfix::check(path, text).map_err(failed)? {
                return Err(WriteError::Recfix {
                    at,
                    path: path.clone(),
                    said,
                    tries,
                });
            }
        }
        let commit = commit::make(
            branch.git_dir(),
            tip.as_ref(),
            &files,
            ask.what,
            ask.who,
            (ask.clock)(),
        )
        .map_err(|not_made| match not_made {
            NotMade::Path { path, reason } => WriteError::Path {
                at: at.clone(),
                path,
                reason,
                tries,
            },
            NotMade::Failed(failure) => failed(failure),
        })?;
        match keeper::move_to(branch, tip.as_ref(), &commit, ask.what).map_err(failed)? {
            Moved::Moved => {
                return Ok(Written::Landed(Landed {
                    commit,
                    on: at,
                    tries,
                }))
            }
            Moved::Refused(said) => {
                let fresh = keeper::read_tip(branch).map_err(failed)?;
                if fresh.as_ref().map(Tip::commit) == at.as_deref() {
                    return Err(WriteError::Failed {
                        step: Step::Move,
                        said,
                    });
                }
                if tries > bound {
                    return Err(WriteError::KeptMoving { tries, bound });
                }
                tip = fresh;
            }
        }
    }
}
