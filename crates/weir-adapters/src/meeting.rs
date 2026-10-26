//! The meeting file: one capture for one meeting file, keyed
//! `meeting/file/<sha256>` so the same bytes under any name key the same,
//! the transcript copied whole into the raw place as its raw material, and
//! no excerpt (62; C.6; core.34). Its event time is the meeting's start
//! where the file states it, in its front matter or on its first line,
//! else the folder drop's (5, 62; C.6). It prints no cursor and ends with
//! its completion (8, 9).

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{Completion, Failure, Line};

use crate::event_time;
use crate::folder::{check_raw_dir, offer_with};

/// The source of the offer the meeting file makes.
pub const SOURCE: &str = "meeting";

const PREFIX: &str = "meeting/file";

/// The lines of one run over `file`, copying into `raw_dir`, or why the
/// run cannot begin: a file or raw place that is not there (9).
pub fn run(file: &Path, raw_dir: &Path) -> Result<Vec<Line>, String> {
    let metadata = file
        .metadata()
        .map_err(|error| format!("the file {} is not there: {error}", file.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a file", file.display()));
    }
    let name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("{} names no file", file.display()))?;
    check_raw_dir(raw_dir)?;
    let mut lines = Vec::new();
    let mut failed = Vec::new();
    let offered = event_at(file).and_then(|event_at| {
        offer_with(
            file,
            &name,
            raw_dir,
            PREFIX,
            SOURCE,
            event_at,
            &mut HashSet::new(),
        )
    });
    match offered {
        Ok(Some(offer)) => lines.push(Line::Offer(offer)),
        Ok(None) => unreachable!("one file's content is never offered twice"),
        Err(why) => failed.push(Failure { what: name, why }),
    }
    lines.push(Line::Done(Completion { listed: 1, failed }));
    Ok(lines)
}

/// The meeting's start where the file states it, else the folder drop's
/// event time (5, 62; C.6).
fn event_at(file: &Path) -> Result<String, String> {
    let start = stated_start(file).map_err(|error| format!("cannot be read: {error}"))?;
    match start {
        Some(start) => Ok(start),
        None => event_time::event_at(file).map_err(|error| format!("cannot be read: {error}")),
    }
}

/// The first the file says of: a `start` field, then a `date` field, of
/// the front matter it opens with, each value the time whole; then a time
/// that opens its first line not blank after any front matter. Only the
/// head of the file is read.
fn stated_start(file: &Path) -> io::Result<Option<String>> {
    let mut reader = BufReader::new(File::open(file)?);
    let mut bytes = Vec::new();
    let mut next = || -> io::Result<Option<String>> {
        bytes.clear();
        if reader.read_until(b'\n', &mut bytes)? == 0 {
            return Ok(None);
        }
        let text = String::from_utf8_lossy(&bytes);
        Ok(Some(text.trim_end_matches(['\n', '\r']).to_string()))
    };
    let mut line = next()?;
    // The front matter: a first line `---`, lines `<name>: <value>`, and a
    // closing `---`. A block never closed is none, and its first line,
    // `---`, states no time.
    if line.as_deref() == Some("---") {
        let mut fields: Vec<(String, String)> = Vec::new();
        loop {
            match next()? {
                None => return Ok(None),
                Some(text) if text == "---" => break,
                Some(text) => {
                    if let Some((name, value)) = text.split_once(':') {
                        fields.push((name.trim().to_string(), value.trim().to_string()));
                    }
                }
            }
        }
        for name in ["start", "date"] {
            let stated = fields
                .iter()
                .rev()
                .find(|(field, _)| field == name)
                .and_then(|(_, value)| stated_time(unquoted(value)));
            if stated.is_some() {
                return Ok(stated);
            }
        }
        line = next()?;
    }
    while line.as_deref().is_some_and(|text| text.trim().is_empty()) {
        line = next()?;
    }
    Ok(line.as_deref().and_then(opening_time))
}

/// `value` with one pair of `"` or `'` around it set aside.
fn unquoted(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner;
        }
    }
    value
}

/// The time `text` states whole: an RFC 3339 time with its offset, kept as
/// written, or else a day `YYYY-MM-DD`, as that day's start in UTC as a
/// name's day is read (5; C.6).
fn stated_time(text: &str) -> Option<String> {
    if OffsetDateTime::parse(text, &Rfc3339).is_ok() {
        return Some(text.to_string());
    }
    event_time::day_start(text)
}

/// The time a line opens with: its first word a time [`stated_time`]
/// reads, or else its first ten characters a day not followed by a digit.
fn opening_time(line: &str) -> Option<String> {
    let line = line.trim_start();
    let word = line.split_whitespace().next()?;
    stated_time(word).or_else(|| {
        let followed_by_digit = line
            .as_bytes()
            .get(10)
            .is_some_and(|byte| byte.is_ascii_digit());
        line.get(..10)
            .filter(|_| !followed_by_digit)
            .and_then(event_time::day_start)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::SystemTime;

    use weir_capture::{content_hash, Offer};

    use super::*;

    fn file(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    fn the_offer(lines: &[Line]) -> &Offer {
        match lines {
            [Line::Offer(offer), Line::Done(_)] => offer,
            _ => panic!("not one offer and a completion: {lines:?}"),
        }
    }

    fn written_at(path: &Path, time: &str) {
        let at = OffsetDateTime::parse(time, &Rfc3339).unwrap();
        File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(SystemTime::from(at))
            .unwrap();
    }

    #[test]
    fn a_transcript_offered() {
        let bytes = b"WEBVTT\n\n00:00.000 --> 00:02.000\nGood morning.\n";
        let (_dir, path) = file("standup.vtt", bytes);
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&path, raw.path()).unwrap();
        let hash = content_hash(bytes);
        let offer = the_offer(&lines);
        assert_eq!(offer.key.as_str(), format!("meeting/file/{hash}"));
        assert_eq!(offer.source, "meeting");
        assert_eq!(offer.title.as_deref(), Some("standup.vtt"));
        assert_eq!(offer.raw, [format!("{hash}/standup.vtt")]);
        assert_eq!(offer.excerpt, None);
        assert_eq!(
            fs::read(raw.path().join(&hash).join("standup.vtt")).unwrap(),
            bytes
        );
        assert_eq!(lines[1].to_json(), r#"{"done":{"listed":1,"failed":[]}}"#);
    }

    #[test]
    fn one_transcript_under_two_names() {
        let (dir, a) = file("a.vtt", b"WEBVTT\n");
        let b = dir.path().join("b.vtt");
        fs::copy(&a, &b).unwrap();
        let raw_a = tempfile::tempdir().unwrap();
        let raw_b = tempfile::tempdir().unwrap();
        let first = run(&a, raw_a.path()).unwrap();
        let second = run(&b, raw_b.path()).unwrap();
        assert_eq!(the_offer(&first).key, the_offer(&second).key);
        assert_eq!(the_offer(&second).title.as_deref(), Some("b.vtt"));
    }

    #[test]
    fn a_file_that_states_its_start() {
        let (_dir, path) = file(
            "call.md",
            b"---\ntitle: standup\nstart: 2026-10-02T14:00:00-04:00\n---\n\nGood morning.\n",
        );
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&path, raw.path()).unwrap();
        assert_eq!(the_offer(&lines).event_at, "2026-10-02T14:00:00-04:00");
    }

    /// The event time of a run over `bytes` named `name`, last written at
    /// 2026-10-03T09:15:42Z.
    fn event_time_of(name: &str, bytes: &[u8]) -> String {
        let (_dir, path) = file(name, bytes);
        written_at(&path, "2026-10-03T09:15:42Z");
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&path, raw.path()).unwrap();
        the_offer(&lines).event_at.clone()
    }

    #[test]
    fn a_date_field_and_no_start() {
        assert_eq!(
            event_time_of(
                "call.md",
                b"---\ntitle: standup\ndate: \"2026-10-02\"\n---\n\nWeekly sync\n"
            ),
            "2026-10-02T00:00:00Z"
        );
        assert_eq!(
            event_time_of("call.md", b"---\ndate: '2026-10-02T09:00:00+01:00'\n---\n"),
            "2026-10-02T09:00:00+01:00"
        );
    }

    #[test]
    fn a_start_before_a_date() {
        assert_eq!(
            event_time_of(
                "call.md",
                b"---\ndate: 2026-10-01\nstart: 2026-10-02T14:00:00Z\n---\n"
            ),
            "2026-10-02T14:00:00Z"
        );
    }

    #[test]
    fn a_start_that_is_not_a_time() {
        assert_eq!(
            event_time_of(
                "2026-10-02-standup.md",
                b"---\nstart: Tuesday\n---\nWeekly sync\n"
            ),
            "2026-10-02T00:00:00Z"
        );
        assert_eq!(
            event_time_of(
                "call.md",
                b"---\nstart: Tuesday\ndate: 2026-10-01\n---\nWeekly sync\n"
            ),
            "2026-10-01T00:00:00Z"
        );
    }

    #[test]
    fn a_date_that_opens_the_first_line() {
        assert_eq!(
            event_time_of("call.txt", b"2026-10-02 Weekly sync\nGood morning.\n"),
            "2026-10-02T00:00:00Z"
        );
        assert_eq!(
            event_time_of("call.txt", b"\n  \n2026-10-02, Weekly sync\n"),
            "2026-10-02T00:00:00Z"
        );
    }

    #[test]
    fn a_time_that_opens_the_first_line_after_the_front_matter() {
        assert_eq!(
            event_time_of(
                "call.md",
                b"---\ntitle: standup\n---\n\n2026-10-02T14:00:00-04:00 Weekly sync\n"
            ),
            "2026-10-02T14:00:00-04:00"
        );
    }

    #[test]
    fn a_transcripts_header() {
        assert_eq!(
            event_time_of(
                "2026-10-02-standup.vtt",
                b"WEBVTT\n\n00:00.000 --> 00:02.000\nGood morning.\n"
            ),
            "2026-10-02T00:00:00Z"
        );
    }

    #[test]
    fn a_field_that_only_begins_with_a_day() {
        assert_eq!(
            event_time_of(
                "call.txt",
                b"---\ndate: 2026-10-02 or so\n---\nWeekly sync\n"
            ),
            "2026-10-03T09:15:42Z"
        );
    }

    #[test]
    fn a_line_that_opens_with_no_time() {
        for bytes in [
            &b"20261002 Weekly sync\n"[..],
            &b"2026-10-021 Weekly sync\n"[..],
            &b"2026-13-02 Weekly sync\n"[..],
            &b"Weekly sync 2026-10-02\n"[..],
            &b"\n\n"[..],
            &b""[..],
        ] {
            assert_eq!(
                event_time_of("call.txt", bytes),
                "2026-10-03T09:15:42Z",
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn a_start_outside_an_opening_block_is_not_read() {
        for bytes in [
            &b"Good morning.\n---\nstart: 2026-10-02T14:00:00Z\n---\n"[..],
            &b"---\nstart: 2026-10-02T14:00:00Z\nno closing line\n"[..],
            &b"---\nstart: 2026-10-02T14:00:00\n---\n"[..],
        ] {
            assert_eq!(event_time_of("call.txt", bytes), "2026-10-03T09:15:42Z");
        }
    }

    #[test]
    fn a_file_that_says_nothing_of_when() {
        assert_eq!(
            event_time_of("call.txt", b"Weekly sync\nGood morning.\n"),
            "2026-10-03T09:15:42Z"
        );
    }

    #[test]
    fn an_unreadable_file() {
        let (_dir, path) = file("call.txt", b"Good morning.\n");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&path, raw.path()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let [Line::Done(done)] = lines.as_slice() else {
            panic!("not a completion alone: {lines:?}");
        };
        assert_eq!(done.listed, 1);
        assert_eq!(done.failed.len(), 1);
        assert_eq!(done.failed[0].what, "call.txt");
        assert!(!done.failed[0].why.is_empty());
        assert_eq!(fs::read_dir(raw.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_run_that_cannot_begin() {
        let dir = tempfile::tempdir().unwrap();
        let raw = tempfile::tempdir().unwrap();
        let why = run(&dir.path().join("gone.vtt"), raw.path()).unwrap_err();
        assert!(why.contains("not there"), "{why}");
        let why = run(dir.path(), raw.path()).unwrap_err();
        assert!(why.contains("not a file"), "{why}");
        let (_file_dir, path) = file("call.txt", b"x");
        let why = run(&path, &dir.path().join("no-raw")).unwrap_err();
        assert!(why.contains("raw place"), "{why}");
    }
}
