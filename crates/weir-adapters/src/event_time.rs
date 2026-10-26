//! A file's event time, read once for every built-in adapter: the day its
//! name begins with, as that day's start in UTC, or else its last write in
//! UTC to the second (5, 61; C.6; core.8). UTC, never the machine's zone, so
//! two machines listing one file give one event time (24; core.14). The
//! reader of crew's folders reads a day alone, a capture's `event_date` and
//! a move's `Date`, with [`day_start`].

use std::fs;
use std::io;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime, UtcOffset};

/// `<day>T00:00:00Z` for `text` when it is `YYYY-MM-DD` naming a day of the
/// calendar, and none otherwise.
pub fn day_start(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let shaped = bytes.len() == 10
        && bytes.iter().enumerate().all(|(n, byte)| match n {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        });
    if !shaped {
        return None;
    }
    let year: i32 = text[0..4].parse().ok()?;
    let month: u8 = text[5..7].parse().ok()?;
    let day: u8 = text[8..10].parse().ok()?;
    Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
    Some(format!("{text}T00:00:00Z"))
}

/// The day the file name `name` begins with, as [`day_start`] reads it.
fn name_day(name: &str) -> Option<String> {
    name.get(..10).and_then(day_start)
}

/// The event time of the file at `path`: the day its name begins with, or
/// else its last write, a symbolic link read as the file it names.
pub fn event_at(path: &Path) -> io::Result<String> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if let Some(day) = name_day(&name) {
        return Ok(day);
    }
    let modified = fs::metadata(path)?.modified()?;
    let utc = OffsetDateTime::from(modified)
        .to_offset(UtcOffset::UTC)
        .replace_nanosecond(0)
        .map_err(io::Error::other)?;
    utc.format(&Rfc3339).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::time::SystemTime;

    use super::*;

    fn written_at(dir: &Path, name: &str, time: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let file = File::create(&path).unwrap();
        let at = OffsetDateTime::parse(time, &Rfc3339).unwrap();
        file.set_modified(SystemTime::from(at)).unwrap();
        path
    }

    #[test]
    fn a_dated_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = written_at(dir.path(), "2026-10-02-standup.vtt", "2026-10-20T11:00:00Z");
        assert_eq!(event_at(&path).unwrap(), "2026-10-02T00:00:00Z");
    }

    #[test]
    fn an_undated_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = written_at(dir.path(), "standup.vtt", "2026-10-03T09:15:42.7Z");
        assert_eq!(event_at(&path).unwrap(), "2026-10-03T09:15:42Z");
        let offset = written_at(dir.path(), "call.txt", "2026-10-03T11:15:42+02:00");
        assert_eq!(event_at(&offset).unwrap(), "2026-10-03T09:15:42Z");
    }

    #[test]
    fn a_name_that_begins_with_no_day() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["2026-13-40-x.txt", "2026-02-30.txt", "2026-1-02-x.txt", "x"] {
            let path = written_at(dir.path(), name, "2026-10-03T09:15:42Z");
            assert_eq!(event_at(&path).unwrap(), "2026-10-03T09:15:42Z", "{name}");
        }
    }

    #[test]
    fn a_day_alone() {
        assert_eq!(
            day_start("2026-10-02").as_deref(),
            Some("2026-10-02T00:00:00Z")
        );
        assert_eq!(
            day_start("2028-02-29").as_deref(),
            Some("2028-02-29T00:00:00Z")
        );
        for text in [
            "2026-02-29",
            "2026-10-02T00:00:00Z",
            "2026-10-2",
            "Tuesday",
            "",
            "２０２６-10-02",
        ] {
            assert_eq!(day_start(text), None, "{text}");
        }
    }

    #[test]
    fn a_file_that_is_not_there() {
        let dir = tempfile::tempdir().unwrap();
        assert!(event_at(&dir.path().join("gone.txt")).is_err());
    }
}
