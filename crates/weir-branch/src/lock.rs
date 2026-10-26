//! The lock that keeps one machine's writers of a branch in turn, and does
//! nothing more: it never stands in for the move from the tip read (49;
//! C.3; core.28).
//!
//! The lock is the file `weir.lock` in the git directory the branch is
//! written through, holding one record: who holds it and when it was taken,
//! by the caller's clock. A writer takes it by writing that record to a
//! file of a unique name beside it and hard-linking that file to
//! `weir.lock`, which fails while one stands, so of two writers taking it
//! at once one alone succeeds. A writer that finds it held looks again
//! every fifty milliseconds until it is let go or older than the stale age
//! the caller sets, and breaks a stale one by renaming it aside and
//! removing it only when it is still the lock judged stale. A writer lets
//! go only a lock whose text is its own.

use std::fmt;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::keeper::{Failure, Step};

/// The lock's file in the git directory.
pub const LOCK: &str = "weir.lock";

/// How long a writer waits before looking at a held lock again.
const LOOK_AGAIN: Duration = Duration::from_millis(50);

/// A stale lock a writer broke: who held it and when it was taken, as its
/// record named them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Broken {
    pub holder: String,
    pub taken_at: String,
}

impl fmt::Display for Broken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the stale lock of {} taken at {}",
            self.holder, self.taken_at
        )
    }
}

/// A lock held: let go when dropped, on every end of a write, a panic
/// among them.
pub(crate) struct Lock {
    dir: PathBuf,
    text: String,
}

impl Drop for Lock {
    fn drop(&mut self) {
        // Nothing reports a lock that could not be let go: the write has
        // ended, and a lock left stands until it is stale.
        let _ = let_go(&self.dir, &self.text);
    }
}

fn failed(said: String) -> Failure {
    Failure::new(Step::Lock, said)
}

/// The lock's record for `holder`, taken at `when`.
fn record(holder: &str, when: OffsetDateTime) -> Result<String, Failure> {
    let taken_at = when
        .format(&Rfc3339)
        .map_err(|error| failed(format!("the clock's time has no RFC 3339 form: {error}")))?;
    Ok(format!("holder: {holder}\ntaken_at: {taken_at}\n"))
}

/// The holder and the time a lock's record names, or why it cannot be read
/// as one.
fn parse(text: &str) -> Result<(String, String, OffsetDateTime), String> {
    let (mut holder, mut taken_at) = (None, None);
    for line in text.lines() {
        match line.split_once(": ") {
            Some(("holder", value)) if holder.is_none() => holder = Some(value),
            Some(("taken_at", value)) if taken_at.is_none() => taken_at = Some(value),
            _ => return Err(format!("the line `{line}` is no field of a lock")),
        }
    }
    let holder = holder.ok_or("it names no holder")?;
    let taken_at = taken_at.ok_or("it names no time it was taken")?;
    let when = OffsetDateTime::parse(taken_at, &Rfc3339)
        .map_err(|_| format!("`{taken_at}` is not an RFC 3339 time"))?;
    Ok((holder.to_string(), taken_at.to_string(), when))
}

/// Takes the lock in `dir` for `holder`, waiting while it is held and
/// breaking it once it is older than `stale_after` by `clock`. Gives the
/// lock held, and the stale lock broken on the way, if any.
pub(crate) fn take(
    dir: &Path,
    holder: &str,
    stale_after: Duration,
    clock: &dyn Fn() -> OffsetDateTime,
) -> Result<(Lock, Option<Broken>), Failure> {
    let path = dir.join(LOCK);
    let mut broke = None;
    loop {
        let text = record(holder, clock())?;
        let mut unique = tempfile::Builder::new()
            .prefix("weir.lock.")
            .tempfile_in(dir)
            .map_err(|error| {
                failed(format!(
                    "no lock could be made in {}: {error}",
                    dir.display()
                ))
            })?;
        unique
            .write_all(text.as_bytes())
            .and_then(|()| unique.flush())
            .map_err(|error| failed(format!("the lock's record not written: {error}")))?;
        match fs::hard_link(unique.path(), &path) {
            Ok(()) => {
                let lock = Lock {
                    dir: dir.to_path_buf(),
                    text,
                };
                return Ok((lock, broke));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(failed(format!(
                    "the lock {} could not be taken: {error}",
                    path.display()
                )))
            }
        }
        drop(unique);

        let held = match fs::read_to_string(&path) {
            Ok(held) => held,
            // Let go since the link failed: take it again.
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(failed(format!(
                    "the lock {} could not be read: {error}",
                    path.display()
                )))
            }
        };
        let (held_by, taken_at, when) = parse(&held).map_err(|reason| {
            failed(format!(
                "the lock {} cannot be read as a lock's record, {reason}: the operator breaks it",
                path.display()
            ))
        })?;
        let age = clock() - when;
        if age.is_positive() && age.unsigned_abs() > stale_after {
            if break_stale(dir, &held)? {
                broke = Some(Broken {
                    holder: held_by,
                    taken_at,
                });
            }
            continue;
        }
        thread::sleep(LOOK_AGAIN);
    }
}

/// Renames the lock aside to a unique name beside it: none when no lock
/// stands. The name aside is removed when it is dropped.
fn set_aside(dir: &Path) -> io::Result<Option<tempfile::TempPath>> {
    let aside = tempfile::Builder::new()
        .prefix("weir.lock.aside.")
        .tempfile_in(dir)?
        .into_temp_path();
    match fs::rename(dir.join(LOCK), &aside) {
        Ok(()) => Ok(Some(aside)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Breaks the stale lock whose text is `judged`, once however many writers
/// judge it stale: true when this writer broke it. A lock found aside that
/// is not the one judged was taken by another writer since, and is linked
/// back.
fn break_stale(dir: &Path, judged: &str) -> Result<bool, Failure> {
    let broken = |error: io::Error| {
        failed(format!(
            "the stale lock {} could not be broken: {error}",
            dir.join(LOCK).display()
        ))
    };
    let Some(aside) = set_aside(dir).map_err(broken)? else {
        return Ok(false);
    };
    let found = fs::read_to_string(&aside).map_err(broken)?;
    if found == judged {
        return Ok(true);
    }
    match fs::hard_link(&aside, dir.join(LOCK)) {
        Ok(()) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(broken(error)),
    }
}

/// Lets go the lock in `dir` when its text is `own`; a lock of another's
/// found there is linked back.
fn let_go(dir: &Path, own: &str) -> io::Result<()> {
    let Some(aside) = set_aside(dir)? else {
        return Ok(());
    };
    if fs::read_to_string(&aside)? != own {
        fs::hard_link(&aside, dir.join(LOCK))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lock_record_reads_back() {
        let when = OffsetDateTime::parse("2026-10-17T06:30:05+02:00", &Rfc3339).unwrap();
        let text = record("operator@one", when).unwrap();
        assert_eq!(
            text,
            "holder: operator@one\ntaken_at: 2026-10-17T06:30:05+02:00\n"
        );
        let (holder, taken_at, read) = parse(&text).unwrap();
        assert_eq!(holder, "operator@one");
        assert_eq!(taken_at, "2026-10-17T06:30:05+02:00");
        assert_eq!(read, when);
    }

    #[test]
    fn a_text_that_is_no_lock_record() {
        for text in [
            "",
            "holder: a\n",
            "taken_at: 2026-10-17T06:30:05Z\n",
            "holder: a\ntaken_at: yesterday\n",
            "holder: a\ntaken_at: 2026-10-17T06:30:05Z\nother: b\n",
            "holder: a\nholder: b\ntaken_at: 2026-10-17T06:30:05Z\n",
        ] {
            assert!(parse(text).is_err(), "{text:?} read as a lock");
        }
    }
}
