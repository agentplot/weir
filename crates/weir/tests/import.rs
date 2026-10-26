//! `weir import signals`, run as a host runs it: the built binary, its
//! standard output read as import lines and its standard error as what it
//! could not carry (weir-command).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use weir_adapters::signals_folder;
use weir_capture::{read_import_line, CameIn};

fn weir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weir"))
        .args(args)
        .output()
        .expect("the weir binary runs")
}

fn import(root: &Path) -> Output {
    weir(&["import", "signals", root.to_str().expect("a UTF-8 path")])
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8(output.stdout.clone())
        .expect("standard output is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn stderr_lines(output: &Output) -> Vec<String> {
    String::from_utf8(output.stderr.clone())
        .expect("standard error is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn capture_md(day: u8) -> String {
    format!(
        "---\nsource: session\nat: 2026-10-0{day}T10:00:00Z\nraw: host-a:~/raw/{day}.jsonl\n---\n\n\
         # The capture of day {day}\n"
    )
}

fn signal_md(kind: &str) -> String {
    format!(
        "---\nkind: {kind}\nwho: design-agent\nsubject: [a, b]\n---\n\nIt is so.\n\n\
         > \"the words\" — 10:00:00Z\n"
    )
}

/// A body of `count` whole captures, `2026-10-0<n>-capture`, each with one
/// signal, and a moves file routing the first one's.
fn body(count: u8) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for day in 1..=count {
        let capture = format!("signals/2026-10-0{day}-capture");
        write(
            dir.path(),
            &format!("{capture}/capture.md"),
            &capture_md(day),
        );
        write(
            dir.path(),
            &format!("{capture}/01-said.md"),
            &signal_md("ask"),
        );
    }
    write(dir.path(), "signals/README.md", "# Signals\n");
    write(
        dir.path(),
        "moves.rec",
        "%rec: Move\n\nSignal: 2026-10-01-capture/01-said\nMove: route\nTarget: unit/x\n\
         Date: 2026-10-02\nBy: planner\n",
    );
    dir
}

#[test]
fn a_body_imported() {
    let root = body(3);
    let output = import(root.path());
    assert_eq!(output.status.code(), Some(0), "{:?}", stderr_lines(&output));
    let lines = stdout_lines(&output);
    assert_eq!(lines.len(), 3);
    for (n, line) in lines.iter().enumerate() {
        let read = read_import_line(line).expect(line);
        assert_eq!(
            read.capture.key.as_str(),
            format!("2026-10-0{}-capture", n + 1)
        );
        assert_eq!(read.came_in, CameIn::Import(signals_folder::BODY.into()));
        assert_eq!(read.signals.len(), 1);
    }
    let first = read_import_line(&lines[0]).unwrap();
    assert_eq!(first.moves.len(), 1);
    assert_eq!(first.moves[0].word, "route");
    assert!(
        stderr_lines(&output).is_empty(),
        "{:?}",
        stderr_lines(&output)
    );
    let files: Vec<_> = fs::read_dir(root.path()).unwrap().collect();
    assert_eq!(files.len(), 2, "nothing written beside the body");
}

#[test]
fn run_twice() {
    let root = body(3);
    let first = import(root.path());
    let second = import(root.path());
    assert_eq!(first.stdout, second.stdout);
    assert!(!first.stdout.is_empty());
}

#[test]
fn one_capture_not_read() {
    let root = body(4);
    write(
        root.path(),
        "signals/2026-10-04-capture/01-said.md",
        &signal_md("finding"),
    );
    let output = import(root.path());
    assert_eq!(stdout_lines(&output).len(), 3);
    let said = stderr_lines(&output);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("2026-10-04-capture"), "{said:?}");
    assert!(said[0].contains("01-said.md"), "{said:?}");
    assert!(said[0].contains("`kind`"), "{said:?}");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_move_not_carried() {
    let root = body(2);
    write(
        root.path(),
        "moves.rec",
        "Signal: 2026-09-30-elsewhere/01-said\nMove: drop\nReason: r\nDate: 2026-10-02\nBy: p\n",
    );
    let output = import(root.path());
    assert_eq!(stdout_lines(&output).len(), 2);
    let said = stderr_lines(&output);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("2026-09-30-elsewhere/01-said"), "{said:?}");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_root_that_cannot_be_read() {
    let dir = tempfile::tempdir().unwrap();
    let output = import(dir.path());
    assert!(output.stdout.is_empty());
    assert_eq!(stderr_lines(&output).len(), 1);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn the_imports_help_comes_from_the_reader() {
    for args in [
        vec![],
        vec!["import"],
        vec!["import", "signals"],
        vec!["import", "signals", "a", "b"],
    ] {
        let output = weir(&args);
        let said = String::from_utf8(output.stderr).unwrap();
        assert!(said.contains("weir import signals <root>"), "{said}");
        assert!(said.contains(signals_folder::HELP), "{said}");
        assert_eq!(output.status.code(), Some(2), "{args:?}");
    }
}
