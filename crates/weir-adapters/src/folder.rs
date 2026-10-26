//! The folder drop: one capture per distinct content among the files
//! directly in a folder, keyed `folder/<the folder's name>/<sha256>`, so a
//! file dropped twice, or under another name, is one capture (61; C.6;
//! core.34). Each file is copied whole into the raw place as its raw
//! material, and no offer carries an excerpt (6). Every run offers every
//! file, prints no cursor and ends with its completion, a file that fails
//! stopping no other (8 to 10; C.1).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use weir_capture::{Completion, Failure, Key, Line, Offer};

use crate::copy::copy;
use crate::event_time::event_at;

/// The source of every offer the folder drop makes.
pub const SOURCE: &str = "folder";

/// The lines of one run over `dir`, copying into `raw_dir`, or why the run
/// cannot begin: a folder or raw place that cannot be read, or a folder
/// whose name the key rule refuses, found before any file is read (4, 9).
pub fn run(dir: &Path, raw_dir: &Path) -> Result<Vec<Line>, String> {
    let folder = dir
        .canonicalize()
        .map_err(|error| format!("the folder {} cannot be read: {error}", dir.display()))?;
    if !folder.is_dir() {
        return Err(format!("{} is not a folder", dir.display()));
    }
    let name = folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("the folder {} has no name to key by", dir.display()))?;
    let prefix = format!("folder/{name}");
    Key::parse(&format!("{prefix}/{}", "0".repeat(64))).map_err(|error| {
        format!("the folder's name `{name}` makes no key the key rule takes: {error}")
    })?;
    check_raw_dir(raw_dir)?;
    let mut entries: Vec<_> = fs::read_dir(&folder)
        .map_err(|error| format!("the folder {} cannot be read: {error}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .filter(|name| !name.as_encoded_bytes().starts_with(b"."))
        .collect();
    entries.sort_by(|a, b| a.as_encoded_bytes().cmp(b.as_encoded_bytes()));

    let mut lines = Vec::new();
    let mut listed = 0;
    let mut failed = Vec::new();
    let mut offered = HashSet::new();
    for entry in entries {
        let path = folder.join(&entry);
        let file_name = entry.to_string_lossy().into_owned();
        // A symbolic link is read as the file it names; a link that names
        // nothing is a file that cannot be read.
        match fs::metadata(&path) {
            Ok(metadata) if !metadata.is_file() => continue,
            _ => {}
        }
        listed += 1;
        match offer_file(&path, &file_name, raw_dir, &prefix, SOURCE, &mut offered) {
            Ok(Some(offer)) => lines.push(Line::Offer(offer)),
            Ok(None) => {}
            Err(why) => failed.push(Failure {
                what: file_name,
                why,
            }),
        }
    }
    lines.push(Line::Done(Completion { listed, failed }));
    Ok(lines)
}

/// The raw place must be a folder that is there: the host hands a fresh
/// one for each run (73).
pub(crate) fn check_raw_dir(raw_dir: &Path) -> Result<(), String> {
    if raw_dir.is_dir() {
        Ok(())
    } else {
        Err(format!(
            "the raw place {} is not a folder",
            raw_dir.display()
        ))
    }
}

/// The offer of the file at `path`, named `name`, keyed `<prefix>/<sha256>`,
/// its bytes copied into the raw place; none when a file of this run had
/// its content already, whose copy is removed; or why it could not be read
/// or copied.
pub(crate) fn offer_file(
    path: &Path,
    name: &str,
    raw_dir: &Path,
    prefix: &str,
    source: &str,
    offered: &mut HashSet<String>,
) -> Result<Option<Offer>, String> {
    let event_at = event_at(path).map_err(|error| format!("cannot be read: {error}"))?;
    offer_with(path, name, raw_dir, prefix, source, event_at, offered)
}

/// As [`offer_file`], with the event time already read.
pub(crate) fn offer_with(
    path: &Path,
    name: &str,
    raw_dir: &Path,
    prefix: &str,
    source: &str,
    event_at: String,
    offered: &mut HashSet<String>,
) -> Result<Option<Offer>, String> {
    let copied =
        copy(path, raw_dir).map_err(|error| format!("cannot be read or copied: {error}"))?;
    let key_text = format!("{prefix}/{}", copied.hash);
    let key = match Key::parse(&key_text) {
        Ok(key) => key,
        Err(error) => {
            copied.discard();
            return Err(format!("makes no key, `{key_text}`: {error}"));
        }
    };
    if !offered.insert(copied.hash.clone()) {
        copied.discard();
        return Ok(None);
    }
    let raw = copied
        .bank(raw_dir, name)
        .map_err(|error| format!("cannot be copied into the raw place: {error}"))?;
    Ok(Some(Offer {
        key,
        source: source.to_string(),
        event_at,
        title: Some(name.to_string()),
        url: None,
        raw: vec![raw],
        excerpt: None,
    }))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs::File;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::PathBuf;
    use std::time::SystemTime;

    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;
    use weir_capture::content_hash;
    use weir_capture::take::{take, Bank, Inbox, RawFile, Refusal, Take, Taken};
    use weir_capture::CameIn;

    use super::*;

    /// A folder named `name` in a temporary directory, holding `files`.
    fn folder(name: &str, files: &[(&str, &[u8])]) -> (tempfile::TempDir, PathBuf) {
        let top = tempfile::tempdir().unwrap();
        let dir = top.path().join(name);
        fs::create_dir(&dir).unwrap();
        for (file, bytes) in files {
            fs::write(dir.join(file), bytes).unwrap();
        }
        (top, dir)
    }

    fn offers(lines: &[Line]) -> Vec<&Offer> {
        lines
            .iter()
            .filter_map(|line| match line {
                Line::Offer(offer) => Some(offer),
                _ => None,
            })
            .collect()
    }

    fn done(lines: &[Line]) -> &Completion {
        match lines.last() {
            Some(Line::Done(done)) => done,
            last => panic!("the last line is no completion: {last:?}"),
        }
    }

    /// An inbox and a bank held in memory, as a host's.
    #[derive(Default)]
    struct Host {
        held: BTreeSet<String>,
        written: Vec<Taken>,
        banked: Vec<(String, Vec<RawFile>)>,
    }

    impl Inbox for Host {
        fn holds(&self, key: &Key) -> bool {
            self.held.contains(key.as_str())
        }

        fn write(&mut self, taken: &Taken) -> Result<(), Refusal> {
            self.held.insert(taken.capture.key.to_string());
            self.written.push(taken.clone());
            Ok(())
        }
    }

    struct HostBank<'a>(&'a mut Vec<(String, Vec<RawFile>)>);

    impl Bank for HostBank<'_> {
        fn bank(
            &mut self,
            key: &Key,
            files: &[RawFile],
            _whole_excerpt: Option<&str>,
        ) -> Result<String, Refusal> {
            self.0.push((key.to_string(), files.to_vec()));
            Ok(format!("/bank/{key}"))
        }

        fn discard(&mut self, _key: &Key) {}
    }

    fn take_lines(host: &mut Host, lines: &[Line], raw_dir: &Path) -> weir_capture::Report {
        let how = Take::new(
            "deneb",
            OffsetDateTime::parse("2026-10-17T04:30:05Z", &Rfc3339).unwrap(),
            8192,
            CameIn::Run("folder".into()),
        )
        .unwrap();
        let lines: Vec<String> = lines.iter().map(Line::to_json).collect();
        let mut banked = std::mem::take(&mut host.banked);
        let report = take(
            &lines,
            Some(raw_dir),
            &how,
            host,
            &mut HostBank(&mut banked),
        );
        host.banked = banked;
        report
    }

    #[test]
    fn a_file_dropped_twice_the_second_time_under_another_name() {
        let (_top, dir) = folder("inbox-drop", &[("notes.txt", b"the notes\n")]);
        let key = format!("folder/inbox-drop/{}", content_hash(b"the notes\n"));
        let first_raw = tempfile::tempdir().unwrap();
        let first = run(&dir, first_raw.path()).unwrap();
        assert_eq!(
            offers(&first)
                .iter()
                .map(|offer| offer.key.as_str())
                .collect::<Vec<_>>(),
            [key.as_str()]
        );

        fs::copy(dir.join("notes.txt"), dir.join("notes-again.txt")).unwrap();
        let second_raw = tempfile::tempdir().unwrap();
        let second = run(&dir, second_raw.path()).unwrap();
        assert_eq!(
            offers(&second)
                .iter()
                .map(|offer| offer.key.as_str())
                .collect::<Vec<_>>(),
            [key.as_str()]
        );
        assert_eq!(done(&second).listed, 2);

        let mut host = Host::default();
        let one = take_lines(&mut host, &first, first_raw.path());
        let two = take_lines(&mut host, &second, second_raw.path());
        assert_eq!(one.taken.len(), 1);
        assert_eq!(two.taken.len(), 0);
        assert_eq!(two.held, 1);
        assert_eq!(host.written.len(), 1);
        assert_eq!(host.written[0].capture.key.as_str(), key);
        assert_eq!(host.banked[0].1[0].bytes, b"the notes\n");
    }

    #[test]
    fn two_files_with_one_content_in_one_run() {
        let (_top, dir) = folder("drop", &[("a.txt", b"same"), ("b.txt", b"same")]);
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        let offers = offers(&lines);
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].title.as_deref(), Some("a.txt"));
        assert_eq!(
            done(&lines),
            &Completion {
                listed: 2,
                failed: vec![]
            }
        );
        // The second copy is removed, and nothing is left under its
        // temporary name.
        let hash = content_hash(b"same");
        let banked: Vec<_> = fs::read_dir(raw.path().join(&hash))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(banked, ["a.txt"]);
        let top: Vec<_> = fs::read_dir(raw.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(top, [hash.as_str()]);
    }

    #[test]
    fn the_same_file_keys_the_same_on_every_run() {
        let (_top, dir) = folder(
            "drop",
            &[
                ("2026-10-02-standup.vtt", b"WEBVTT\n"),
                ("notes.md", b"# n\n"),
            ],
        );
        let one = tempfile::tempdir().unwrap();
        let two = tempfile::tempdir().unwrap();
        let first: Vec<String> = run(&dir, one.path())
            .unwrap()
            .iter()
            .map(Line::to_json)
            .collect();
        let second: Vec<String> = run(&dir, two.path())
            .unwrap()
            .iter()
            .map(Line::to_json)
            .collect();
        assert_eq!(first.len(), 3);
        assert_eq!(first, second);
    }

    #[test]
    fn hidden_files_and_subfolders() {
        let (_top, dir) = folder(
            "drop",
            &[(".DS_Store", b"\0\0"), ("report.pdf", b"%PDF-1.7\n")],
        );
        fs::create_dir(dir.join("old")).unwrap();
        fs::write(dir.join("old").join("inner.txt"), b"inner").unwrap();
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        let offers = offers(&lines);
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].title.as_deref(), Some("report.pdf"));
        assert_eq!(done(&lines).listed, 1);
    }

    #[test]
    fn a_symbolic_link_is_read_as_the_file_it_names() {
        let (top, dir) = folder("drop", &[]);
        let outside = top.path().join("elsewhere.txt");
        fs::write(&outside, b"linked bytes").unwrap();
        symlink(&outside, dir.join("link.txt")).unwrap();
        symlink(top.path(), dir.join("a-folder")).unwrap();
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        let offers = offers(&lines);
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].title.as_deref(), Some("link.txt"));
        let hash = content_hash(b"linked bytes");
        assert_eq!(offers[0].raw, [format!("{hash}/link.txt")]);
        assert_eq!(
            fs::read(raw.path().join(&hash).join("link.txt")).unwrap(),
            b"linked bytes"
        );
        assert_eq!(done(&lines).listed, 1);
    }

    #[test]
    fn the_raw_copy() {
        let bytes = b"line one\r\nline two\n\x00\xff";
        let (_top, dir) = folder("drop", &[("notes.txt", bytes)]);
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        let offers = offers(&lines);
        let hash = content_hash(bytes);
        assert_eq!(offers[0].raw, [format!("{hash}/notes.txt")]);
        assert_eq!(offers[0].source, "folder");
        assert_eq!(offers[0].excerpt, None);
        assert_eq!(
            fs::read(raw.path().join(&hash).join("notes.txt")).unwrap(),
            bytes
        );
        assert!(!lines[0].to_json().contains("excerpt"));
    }

    #[test]
    fn the_event_time_is_the_names_day_or_else_the_last_write() {
        let (_top, dir) = folder(
            "drop",
            &[("2026-10-02-standup.vtt", b"a"), ("standup.vtt", b"b")],
        );
        let at = OffsetDateTime::parse("2026-10-03T09:15:42.7Z", &Rfc3339).unwrap();
        File::options()
            .write(true)
            .open(dir.join("standup.vtt"))
            .unwrap()
            .set_modified(SystemTime::from(at))
            .unwrap();
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        let times: Vec<&str> = offers(&lines)
            .iter()
            .map(|offer| offer.event_at.as_str())
            .collect();
        assert_eq!(times, ["2026-10-02T00:00:00Z", "2026-10-03T09:15:42Z"]);
    }

    #[test]
    fn one_file_unreadable() {
        let (_top, dir) = folder("drop", &[("a.txt", b"a"), ("b.txt", b"b"), ("c.txt", b"c")]);
        fs::set_permissions(dir.join("b.txt"), fs::Permissions::from_mode(0o000)).unwrap();
        let raw = tempfile::tempdir().unwrap();
        let lines = run(&dir, raw.path()).unwrap();
        fs::set_permissions(dir.join("b.txt"), fs::Permissions::from_mode(0o644)).unwrap();
        let titles: Vec<_> = offers(&lines)
            .iter()
            .map(|offer| offer.title.as_deref().unwrap())
            .collect();
        assert_eq!(titles, ["a.txt", "c.txt"]);
        let done = done(&lines);
        assert_eq!(done.listed, 3);
        assert_eq!(done.failed.len(), 1);
        assert_eq!(done.failed[0].what, "b.txt");
        assert!(!done.failed[0].why.is_empty());
        assert!(!raw.path().join(".copying").exists());
    }

    #[test]
    fn a_folder_whose_name_the_key_rule_refuses() {
        let (_top, dir) = folder("capture.rec", &[("a.txt", b"a")]);
        let raw = tempfile::tempdir().unwrap();
        let why = run(&dir, raw.path()).unwrap_err();
        assert!(why.contains("`capture.rec`"), "{why}");
        assert_eq!(fs::read_dir(raw.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_run_that_cannot_begin() {
        let top = tempfile::tempdir().unwrap();
        let raw = tempfile::tempdir().unwrap();
        let why = run(&top.path().join("gone"), raw.path()).unwrap_err();
        assert!(why.contains("cannot be read"), "{why}");
        let (_dir_top, dir) = folder("drop", &[("a.txt", b"a")]);
        let why = run(&dir, &top.path().join("no-raw")).unwrap_err();
        assert!(why.contains("raw place"), "{why}");
        let why = run(&dir.join("a.txt"), raw.path()).unwrap_err();
        assert!(why.contains("not a folder"), "{why}");
    }
}
