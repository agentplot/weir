//! `weir capture folder` and `weir capture meeting`, run as a host runs
//! any adapter: the built binary, its standard output read as the
//! contract's lines (weir-command).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{take, Bank, Inbox, RawFile, Refusal, Take, Taken};
use weir_capture::{content_hash, read_line, CameIn, Completion, Key, Line};

fn weir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weir"))
        .args(args)
        .output()
        .expect("the weir binary runs")
}

fn path(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8(output.stdout.clone())
        .expect("standard output is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("standard error is UTF-8")
}

/// A folder named `name` in a temporary directory, holding `files`.
fn folder(name: &str, files: &[(&str, &[u8])]) -> (tempfile::TempDir, std::path::PathBuf) {
    let top = tempfile::tempdir().unwrap();
    let dir = top.path().join(name);
    fs::create_dir(&dir).unwrap();
    for (file, bytes) in files {
        fs::write(dir.join(file), bytes).unwrap();
    }
    (top, dir)
}

#[test]
fn the_folder_drop_through_the_command() {
    let (_top, dir) = folder("drop", &[("a.txt", b"a"), ("b.txt", b"b")]);
    let raw = tempfile::tempdir().unwrap();
    let output = weir(&[
        "capture",
        "folder",
        path(&dir),
        "--raw-dir",
        path(raw.path()),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines: Vec<Line> = stdout_lines(&output)
        .iter()
        .map(|line| read_line(line).expect(line))
        .collect();
    assert_eq!(lines.len(), 3);
    assert!(matches!(lines[0], Line::Offer(_)));
    assert!(matches!(lines[1], Line::Offer(_)));
    assert_eq!(
        lines[2],
        Line::Done(Completion {
            listed: 2,
            failed: vec![]
        })
    );
}

#[test]
fn the_meeting_file_through_the_command() {
    let (_top, dir) = folder("calls", &[("standup.vtt", b"WEBVTT\n")]);
    let raw = tempfile::tempdir().unwrap();
    let output = weir(&[
        "capture",
        "meeting",
        path(&dir.join("standup.vtt")),
        "--raw-dir",
        path(raw.path()),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = stdout_lines(&output);
    let Ok(Line::Offer(offer)) = read_line(&lines[0]) else {
        panic!("not an offer: {lines:?}");
    };
    assert_eq!(
        offer.key.as_str(),
        format!("meeting/file/{}", content_hash(b"WEBVTT\n"))
    );
    assert_eq!(lines[1], r#"{"done":{"listed":1,"failed":[]}}"#);
}

/// An inbox and a raw bank held in memory, as a host's.
#[derive(Default)]
struct Host {
    held: BTreeSet<String>,
    written: Vec<Taken>,
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

#[derive(Default)]
struct HostBank {
    banked: Vec<(String, Vec<RawFile>)>,
}

impl Bank for HostBank {
    fn bank(&mut self, key: &Key, files: &[RawFile], _: Option<&str>) -> Result<String, Refusal> {
        self.banked.push((key.to_string(), files.to_vec()));
        Ok(format!("/bank/{key}"))
    }

    fn discard(&mut self, _: &Key) {}
}

#[test]
fn a_hosts_take_of_the_commands_lines() {
    let (_top, dir) = folder("inbox-drop", &[("notes.txt", b"the notes\n")]);
    let how = Take::new(
        "deneb",
        OffsetDateTime::parse("2026-10-17T04:30:05Z", &Rfc3339).unwrap(),
        8192,
        CameIn::Run("folder".into()),
    )
    .unwrap();
    let mut host = Host::default();
    let mut bank = HostBank::default();
    let mut reports = Vec::new();
    for second in [false, true] {
        if second {
            fs::copy(dir.join("notes.txt"), dir.join("notes-again.txt")).unwrap();
        }
        let raw = tempfile::tempdir().unwrap();
        let output = weir(&[
            "capture",
            "folder",
            path(&dir),
            "--raw-dir",
            path(raw.path()),
        ]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        reports.push(take(
            stdout_lines(&output),
            Some(raw.path()),
            &how,
            &mut host,
            &mut bank,
        ));
    }
    assert_eq!(reports[0].taken.len(), 1);
    assert_eq!(reports[1].taken.len(), 0);
    assert_eq!(reports[1].held, 1);
    assert_eq!(host.written.len(), 1, "one capture");
    assert_eq!(
        host.written[0].capture.key.as_str(),
        format!("folder/inbox-drop/{}", content_hash(b"the notes\n"))
    );
    assert_eq!(bank.banked.len(), 1);
    assert_eq!(bank.banked[0].1[0].bytes, b"the notes\n");
}

#[test]
fn no_raw_place() {
    let (_top, dir) = folder("calls", &[("call.txt", b"hello\n")]);
    let output = weir(&["capture", "meeting", path(&dir.join("call.txt"))]);
    assert!(output.stdout.is_empty());
    let said = stderr(&output);
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.contains("--raw-dir"), "{said}");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_cursor_handed_back() {
    let (_top, dir) = folder("drop", &[("a.txt", b"a"), ("2026-10-02-b.txt", b"b")]);
    let one = tempfile::tempdir().unwrap();
    let two = tempfile::tempdir().unwrap();
    let without = weir(&[
        "capture",
        "folder",
        path(&dir),
        "--raw-dir",
        path(one.path()),
    ]);
    let with = weir(&[
        "capture",
        "folder",
        path(&dir),
        "--raw-dir",
        path(two.path()),
        "--cursor",
        "c1",
    ]);
    assert_eq!(with.status.code(), Some(0), "{}", stderr(&with));
    assert_eq!(stdout_lines(&with), stdout_lines(&without));
    assert!(stdout_lines(&with)
        .iter()
        .all(|line| !matches!(read_line(line), Ok(Line::Cursor(_)))));
}

#[test]
fn a_run_that_cannot_begin_prints_no_completion() {
    let top = tempfile::tempdir().unwrap();
    let raw = tempfile::tempdir().unwrap();
    let (_refused_top, refused) = folder("capture.rec", &[("a.txt", b"a")]);
    for args in [
        vec![
            "capture",
            "folder",
            path(&top.path().join("gone")),
            "--raw-dir",
            path(raw.path()),
        ],
        vec![
            "capture",
            "meeting",
            path(&top.path().join("gone.vtt")),
            "--raw-dir",
            path(raw.path()),
        ],
        vec![
            "capture",
            "folder",
            path(&refused),
            "--raw-dir",
            path(raw.path()),
        ],
    ] {
        let output = weir(&args);
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(stderr(&output).lines().count(), 1, "{args:?}");
        assert_eq!(output.status.code(), Some(1), "{args:?}");
    }
}

#[test]
fn an_unknown_command() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["capture", "nothing"],
        vec![],
        vec!["capture"],
        vec!["capture", "folder"],
        vec!["capture", "folder", path(dir.path()), "--raw-dir"],
        vec!["capture", "folder", path(dir.path()), "--verbose"],
        vec!["capture", "folder", path(dir.path()), path(dir.path())],
    ] {
        let output = weir(&args);
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(stderr(&output).starts_with("usage:"), "{args:?}");
        assert!(
            stderr(&output).contains("weir capture folder <dir>"),
            "{args:?}"
        );
        assert_eq!(output.status.code(), Some(2), "{args:?}");
    }
}
