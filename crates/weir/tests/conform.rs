//! `weir conform`, run as a host's tests run it: the built binary against
//! the example host, `examples/pack-host.rs`, and against a host that only
//! records how it was run, held to the scenarios of host-conformance (65,
//! 87; C.7; S15).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn weir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weir"))
        .args(args)
        .output()
        .expect("the weir binary runs")
}

/// The example host, built beside the command.
fn example_host() -> PathBuf {
    let host = Path::new(env!("CARGO_BIN_EXE_weir"))
        .parent()
        .unwrap()
        .join("examples/pack-host");
    assert!(host.is_file(), "{} is not built", host.display());
    host
}

/// The repository's pack.
fn the_pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack")
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

/// `weir conform --pack <the repository's pack>` against the example
/// host, with the fault `fault` switched on where one is named: its exit
/// status, and each case's line by its name.
fn conform(fault: Option<&str>) -> (Option<i32>, BTreeMap<String, String>) {
    let bank = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_weir"));
    command
        .args([
            "conform",
            "--pack",
            text(&the_pack()),
            text(&example_host()),
        ])
        .env("PACK_HOST_BANK", bank.path());
    if let Some(fault) = fault {
        command.env(fault, "1");
    }
    let output = command.output().unwrap();
    let lines = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (name, _) = line
                .split_once(": ")
                .expect("a case's name, then its verdict");
            (name.to_string(), line.to_string())
        })
        .collect();
    (output.status.code(), lines)
}

/// The host cases the pack holds at least (host-conformance).
const NAMED: [&str; 8] = [
    "an-own-excerpt-cut-at-the-limit",
    "the-same-offer-held",
    "fourteen-offers-one-under-moves",
    "a-run-without-its-completion",
    "a-malformed-line-among-good-ones",
    "raw-material-at-an-intake-with-no-raw-place",
    "a-secret-in-a-field-the-contract-does-not-name",
    "an-import-written-as-read-and-again",
];

#[test]
fn a_host_that_keeps_the_contract() {
    let (code, lines) = conform(None);
    for name in NAMED {
        assert!(lines.contains_key(name), "no case {name}: {lines:#?}");
    }
    let failed: Vec<&String> = lines
        .values()
        .filter(|line| !line.ends_with(": passed"))
        .collect();
    assert_eq!(failed, Vec::<&String>::new());
    assert_eq!(code, Some(0));
}

#[test]
fn a_field_of_the_hosts_own() {
    // Two runs of a host that keeps the contract, at different times and
    // with banks of their own, both pass.
    let (first, _) = conform(None);
    let (second, _) = conform(None);
    assert_eq!((first, second), (Some(0), Some(0)));
}

/// Runs the pack with `fault` on, and finds the case it breaks failed
/// naming its clauses, and every other case run.
fn fails(fault: &str, case: &str, cites: &str, says: &str) {
    let (code, lines) = conform(Some(fault));
    assert_eq!(code, Some(1), "{fault}: {lines:#?}");
    for name in NAMED {
        assert!(lines.contains_key(name), "{fault}: no line for {name}");
    }
    let line = &lines[case];
    assert!(
        line.starts_with(&format!("{case}: failed, citing {cites}")),
        "{fault}: {line}"
    );
    assert!(line.contains(says), "{fault}: {line}");
}

#[test]
fn a_host_keeps_raw_material_in_a_record() {
    fails(
        "PACK_HOST_RAW_IN_RECORD",
        "an-own-excerpt-cut-at-the-limit",
        "6, 8, 9, 11, 16, 17, 18",
        "inbox/captures/release/acme/1.0/capture.rec differs from tip/'s at line 12: it holds `raw: line 1 of the acme 1.0 release notes",
    );
    let (_, lines) = conform(Some("PACK_HOST_RAW_IN_RECORD"));
    assert!(lines["a-malformed-line-among-good-ones"].ends_with(": passed"));
}

#[test]
fn a_host_keeps_raw_material_beside_its_record() {
    fails(
        "PACK_HOST_RAW_BESIDE",
        "an-own-excerpt-cut-at-the-limit",
        "6, 8, 9, 11, 16, 17",
        "the tip holds inbox/captures/release/acme/1.0/notes.md, which tip/ does not",
    );
}

#[test]
fn a_run_whose_cursor_is_kept_from_an_incomplete_run() {
    fails(
        "PACK_HOST_CURSOR_FROM_INCOMPLETE",
        "a-run-without-its-completion",
        "8, 9, 14, 23",
        "step 1: it kept the cursor `c-before-the-end`, and the case expects none",
    );
}

#[test]
fn a_refusal_with_another_reason() {
    fails(
        "PACK_HOST_REFUSAL_REASON",
        "fourteen-offers-one-under-moves",
        "4, 13, 15, 19, 22, 23",
        "it refused line 7 for `moves/acme` because `refused`, and the case expects a reason \
         holding `a key's first segment is never `moves``",
    );
}

#[test]
fn a_host_that_prints_no_report() {
    fails(
        "PACK_HOST_NO_REPORT",
        "a-secret-in-a-field-the-contract-does-not-name",
        "2, 13, 71",
        "step 1: the host exited 1 and printed no report, saying `pack-host: no report",
    );
    let (_, lines) = conform(Some("PACK_HOST_NO_REPORT"));
    assert!(lines
        .values()
        .all(|line| line.contains(": failed, citing ")));
}

#[test]
fn no_host_command() {
    for args in [
        &["conform"][..],
        &["conform", "--pack"],
        &["conform", "--pack", "pack"],
    ] {
        let output = weir(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("weir conform [--pack <dir>] <host command> [<arg>...]"),
            "{stderr}"
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn a_pack_that_is_not_there() {
    let output = weir(&["conform", "--pack", "/nonexistent/pack", "true"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("weir conform: the pack's"), "{stderr}");
}

/// A host that records how it was run, each call a folder of `record`:
/// its words one a line, its standard input, the declaration a run names,
/// and the files of the branch it was handed; then it prints a report
/// taking nothing.
const RECORDING_HOST: &str = r#"#!/bin/sh
set -eu
call="$RECORD/$(ls "$RECORD" | wc -l | tr -d ' ')"
mkdir "$call"
for arg in "$@"; do
  printf '%s\n' "$arg" >>"$call/args"
done
cat >"$call/stdin"
inbox=
while [ $# -gt 0 ]; do
  if [ "$1" = "--inbox" ]; then
    inbox=$2
  fi
  if [ "$1" = "--run" ]; then
    cp "$2" "$call/declaration"
  fi
  shift
done
printf '%s\n' "$inbox" >"$call/inbox"
git --git-dir "$inbox" ls-tree -r --name-only inbox >"$call/tip"
echo "talk before the report"
echo '{"format":"weir-report/1","taken":[],"held":0,"refused":[],"failed":[],"complete":false}'
"#;

const CAPTURE: &str = "format: weir-capture/1\nkey: release/acme/1.0\nsource: release\n";

/// A pack of three cases, one of each invocation, written into `dir`.
fn three_cases(dir: &Path) {
    let files = [
        (
            "host/a-run/case.toml",
            "satisfies = [23]\nlimit = 4096\n\n[[step]]\nrun = \"adapter.sh\"\nlines = \"lines.jsonl\"\ntaken = []\nheld = 0\ncomplete = false\n",
        ),
        ("host/a-run/adapter.sh", "#!/bin/sh\ncat \"$(dirname \"$0\")/lines.jsonl\"\n"),
        ("host/a-run/lines.jsonl", "{\"cursor\": \"c1\"}\n"),
        (
            "host/an-import/case.toml",
            "satisfies = [37]\nlimit = 4096\n\n[[step]]\nimport = \"import.jsonl\"\ntaken = []\nheld = 0\n",
        ),
        ("host/an-import/import.jsonl", "{\"import\": {}}\n"),
        (
            "host/an-intake/case.toml",
            "satisfies = [14]\nlimit = 4096\n\n[[step]]\nintake = \"lines.jsonl\"\nraw = \"raw\"\ntaken = []\nheld = 0\n",
        ),
        ("host/an-intake/lines.jsonl", "{\"cursor\": \"c2\"}\n"),
        ("host/an-intake/raw/notes.md", "notes\n"),
        (
            "host/an-intake/start/inbox/captures/release/acme/1.0/capture.rec",
            CAPTURE,
        ),
        (
            "host/an-intake/tip/inbox/captures/release/acme/1.0/capture.rec",
            CAPTURE,
        ),
    ];
    for (path, text) in files {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }
}

/// Each call the recording host recorded, in order: its words, its
/// standard input, the inbox it was handed and the files at its branch's
/// tip.
fn calls(record: &Path) -> Vec<(Vec<String>, String, PathBuf, String)> {
    let mut calls = Vec::new();
    for n in 0.. {
        let call = record.join(n.to_string());
        if !call.exists() {
            break;
        }
        let read = |name: &str| fs::read_to_string(call.join(name)).unwrap();
        calls.push((
            read("args").lines().map(str::to_string).collect(),
            read("stdin"),
            PathBuf::from(read("inbox").trim()),
            read("tip"),
        ));
    }
    calls
}

#[test]
fn each_step_is_one_of_three_invocations_with_the_cases_flags() {
    let pack = tempfile::tempdir().unwrap();
    three_cases(pack.path());
    let record = tempfile::tempdir().unwrap();
    let host = tempfile::tempdir().unwrap();
    let script = host.path().join("host-under-test");
    fs::write(&script, RECORDING_HOST).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_weir"))
        .args([
            "conform",
            "--pack",
            text(pack.path()),
            text(&script),
            "--flag",
        ])
        .env("RECORD", record.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    // The report read is the last line the host printed, after its talk;
    // each case expects the report it prints and the tip it leaves, the
    // intake's its start.
    assert_eq!(
        stdout,
        "a-run: passed\nan-import: passed\nan-intake: passed\n"
    );
    assert_eq!(output.status.code(), Some(0));

    let calls = calls(record.path());
    assert_eq!(calls.len(), 3, "{calls:#?}");
    let flags = |args: &[String], inbox: &Path| {
        let head: Vec<&str> = args.iter().take(7).map(String::as_str).collect();
        assert_eq!(
            head,
            [
                "--flag",
                "--inbox",
                text(inbox),
                "--branch",
                "inbox",
                "--limit",
                "4096"
            ]
        );
    };

    // The run: its declaration and no lines on standard input.
    let (args, stdin, inbox, tip) = &calls[0];
    flags(args, inbox);
    assert_eq!(args[7], "--run");
    assert_eq!(args.len(), 9);
    assert_eq!(stdin, "");
    assert_eq!(tip, "");
    let declaration = fs::read_to_string(record.path().join("0/declaration")).unwrap();
    assert!(
        declaration.starts_with("[adapter.\"a-run\"]\ncommand = [\""),
        "{declaration}"
    );
    assert!(
        declaration.contains("/case/adapter.sh\"]\n"),
        "{declaration}"
    );
    assert!(declaration.contains("config = \"config.schema.json\"\nsecrets = []\n"));

    // The import: `--import`, its import lines on standard input.
    let (args, stdin, inbox, _) = &calls[1];
    flags(args, inbox);
    assert_eq!(&args[7..], ["--import"]);
    assert_eq!(stdin, "{\"import\": {}}\n");

    // The intake: its raw place, its lines on standard input, and a branch
    // whose tip already holds the case's start.
    let (args, stdin, inbox, tip) = &calls[2];
    flags(args, inbox);
    assert_eq!(args[7], "--raw-dir");
    assert!(args[8].ends_with("/case/raw"), "{}", args[8]);
    assert_eq!(stdin, "{\"cursor\": \"c2\"}\n");
    assert_eq!(tip, "inbox/captures/release/acme/1.0/capture.rec\n");

    // Each case had a bare repository of its own, removed when it ended.
    assert_ne!(calls[0].2, calls[2].2);
    for (_, _, inbox, _) in &calls {
        assert!(!inbox.exists(), "{} is left behind", inbox.display());
    }
}
