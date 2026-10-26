//! `weir check` reads a declaration: the built binary run against the kind
//! format's fixtures, each fixture's bytes the same after as before (58).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fixture of the kind format's crate.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../weir-kinds/tests/fixtures")
        .join(name)
}

/// `weir check <path>`, run on a file that is the same bytes after it.
fn check(path: &Path) -> Output {
    let before = std::fs::read(path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weir"))
        .arg("check")
        .arg(path)
        .output()
        .unwrap();
    assert_eq!(std::fs::read(path).unwrap(), before, "{}", path.display());
    output
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

#[test]
fn a_declaration_that_reads() {
    let output = check(&fixture("fix-and-change.toml"));
    assert_eq!(
        text(&output.stdout),
        "change@1\n  construct: change\n  code: tasks\n  verify: archived\n\
         fix@1\n  code: commits\n"
    );
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_conversation_and_an_elaboration_kind_are_listed() {
    let output = check(&fixture("bump.toml"));
    assert_eq!(
        text(&output.stdout),
        "bump@2\n  code: commits\n  review: conversation\n"
    );
    assert_eq!(output.status.code(), Some(0));

    let output = check(&fixture("elaboration.toml"));
    assert_eq!(
        text(&output.stdout),
        "writeback@1\n  write: committed:books/conclusion.md\n  elaboration: self-closing\n"
    );
    assert_eq!(output.status.code(), Some(0));

    let output = check(&fixture("endings.toml"));
    assert!(text(&output.stdout).contains("open@1\n  write: commits\n  elaboration: no ending\n"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_declaration_that_is_refused() {
    let output = check(&fixture("two-deliverables.toml"));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        "kinds.bump.stages[0].deliverables: names 2 deliverables, and a stage has exactly one\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn every_problem_of_the_kinds_is_named() {
    let output = check(&fixture("two-faults.toml"));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        "kinds.bare.stages: is missing, and a kind has at least one stage\n\
         kinds.bump.stages[1].prompt: is missing, and every stage has a prompt\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn an_elaboration_section_is_checked_too() {
    let output = check(&fixture("elaboration-shapes.toml"));
    assert_eq!(text(&output.stdout), "");
    let fields: Vec<_> = text(&output.stderr)
        .lines()
        .map(|line| line.split_once(": ").unwrap().0)
        .collect();
    assert_eq!(
        fields,
        [
            "kinds.scalar.elaboration",
            "kinds.shapes.elaboration.inputs",
            "kinds.shapes.elaboration.conclusion",
            "kinds.shapes.elaboration.ending",
            "kinds.shapes.elaboration.transcripts",
            "kinds.shapes.elaboration.lands",
        ]
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn text_that_is_not_toml_is_refused_where_it_fails() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("not-toml.toml");
    std::fs::write(&path, "[[kinds.bump.stages]]\nname = \"code\"\nprompt = \n").unwrap();
    let output = check(&path);
    assert_eq!(text(&output.stdout), "");
    let stderr = text(&output.stderr);
    assert!(stderr.starts_with("line 3, column 10: "), "{stderr}");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_file_it_cannot_read_is_named() {
    let path = fixture("no-such-declaration.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_weir"))
        .arg("check")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(text(&output.stdout), "");
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with(&format!("weir check: {}: ", path.display())),
        "{stderr}"
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!path.exists());
}
