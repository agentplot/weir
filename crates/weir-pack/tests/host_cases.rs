//! The host case format, held to "A host case holds its steps, its
//! starting records and the records expected" (host-conformance; 15, 23;
//! C.7).

use weir_pack::{read_host_case, Clause, ExpectedRun, HostCase, Invocation, Pack, Shape};

const SCRIPT: &str = "#!/bin/sh\ncat \"$(dirname \"$0\")/lines.jsonl\"\n";
const CAPTURE: &str = "format: weir-capture/1\nkey: release/acme/1.0\n";

/// The host case `name` of a pack of `files`, each `(path, text)` from
/// the case's folder.
fn read(files: &[(&str, &str)]) -> Result<HostCase, String> {
    let pack = Pack::from_files(
        files
            .iter()
            .map(|(path, text)| (format!("host/made/{path}"), text.as_bytes().to_vec())),
    );
    read_host_case(&pack.case(Shape::Host, "made").unwrap())
}

const TWO_RUNS: &str = r#"satisfies = [8, 9, 14, "75a"]
limit = 8192

[[step]]
run = "first.sh"
lines = "lines.jsonl"
raw = "raw"
taken = ["release/acme/1.1"]
held = 1
complete = false

[[step.refused]]
line = 3
key = "moves/x"
reason_holds = "first segment"

[[step]]
run = "again.sh"
lines = "lines.jsonl"
taken = []
held = 2
listed = 3
cursor = "c2"
complete = true

[[step.failed]]
what = "github:acme"
why = "HTTP 502"
"#;

#[test]
fn a_case_read() {
    let case = read(&[
        ("case.toml", TWO_RUNS),
        ("first.sh", SCRIPT),
        ("again.sh", SCRIPT),
        ("lines.jsonl", "{\"cursor\": \"c1\"}\n"),
        ("raw/notes.md", "notes\n"),
        ("start/inbox/captures/release/acme/1.0/capture.rec", CAPTURE),
        ("tip/inbox/captures/release/acme/1.0/capture.rec", CAPTURE),
    ])
    .unwrap();
    assert_eq!(case.id, "host/made");
    assert_eq!(
        case.satisfies,
        [8, 9, 14]
            .map(Clause::number)
            .into_iter()
            .chain(Clause::parse("75a"))
            .collect::<Vec<_>>()
    );
    assert_eq!(case.limit, 8192);
    assert_eq!(case.steps.len(), 2);
    assert_eq!(
        case.steps[0].invocation,
        Invocation::Run {
            adapter: "first.sh".into(),
            lines: "lines.jsonl".into(),
            raw: Some("raw".into())
        }
    );
    assert_eq!(case.steps[0].expected.taken, ["release/acme/1.1"]);
    assert_eq!(case.steps[0].expected.held, 1);
    assert_eq!(case.steps[0].expected.refused[0].line, 3);
    assert_eq!(
        case.steps[0].expected.refused[0].key.as_deref(),
        Some("moves/x")
    );
    assert_eq!(
        case.steps[0].expected.run,
        Some(ExpectedRun {
            listed: None,
            failed: vec![],
            cursor: None,
            complete: false
        })
    );
    let again = case.steps[1].expected.run.as_ref().unwrap();
    assert_eq!(again.listed, Some(3));
    assert_eq!(again.cursor.as_deref(), Some("c2"));
    assert_eq!(again.failed[0].why, "HTTP 502");
    assert_eq!(
        case.start.keys().collect::<Vec<_>>(),
        ["inbox/captures/release/acme/1.0/capture.rec"]
    );
    assert_eq!(
        case.tip.keys().collect::<Vec<_>>(),
        case.start.keys().collect::<Vec<_>>()
    );
}

#[test]
fn an_intake_and_an_import_read() {
    let case = read(&[
        (
            "case.toml",
            "satisfies = [13]\nlimit = 8192\n\n[[step]]\nintake = \"lines.jsonl\"\ntaken = []\nheld = 0\n\n[[step]]\nimport = \"import.jsonl\"\ntaken = []\nheld = 0\n",
        ),
        ("lines.jsonl", "\n"),
        ("import.jsonl", "\n"),
    ])
    .unwrap();
    assert_eq!(
        case.steps[0].invocation,
        Invocation::Intake {
            lines: "lines.jsonl".into(),
            raw: None
        }
    );
    assert_eq!(
        case.steps[1].invocation,
        Invocation::Import {
            lines: "import.jsonl".into()
        }
    );
    assert!(case.steps.iter().all(|step| step.expected.run.is_none()));
    assert!(case.start.is_empty());
}

#[test]
fn a_misspelt_field() {
    let misspelt = TWO_RUNS.replace("[[step.refused]]", "[[step.refusals]]");
    let refused = read(&[
        ("case.toml", &misspelt),
        ("first.sh", SCRIPT),
        ("again.sh", SCRIPT),
        ("lines.jsonl", "\n"),
        ("raw/notes.md", "notes\n"),
    ])
    .unwrap_err();
    assert!(refused.starts_with("host/made: "), "{refused}");
    assert!(refused.contains("`refusals`"), "{refused}");
}

#[test]
fn a_step_out_of_its_shape() {
    let head = "satisfies = [13]\nlimit = 8192\n\n[[step]]\n";
    for (step, says) in [
        ("taken = []\nheld = 0\n", "names no invocation"),
        (
            "intake = \"lines.jsonl\"\nimport = \"lines.jsonl\"\ntaken = []\nheld = 0\n",
            "more than one",
        ),
        (
            "intake = \"lines.jsonl\"\ntaken = []\nheld = 0\ncomplete = true\n",
            "`complete` is a run's alone",
        ),
        (
            "run = \"first.sh\"\ntaken = []\nheld = 0\ncomplete = true\n",
            "`lines` is missing",
        ),
        (
            "run = \"first.sh\"\nlines = \"lines.jsonl\"\ntaken = []\nheld = 0\n",
            "`complete` is missing",
        ),
        (
            "intake = \"gone.jsonl\"\ntaken = []\nheld = 0\n",
            "`gone.jsonl`, which is no file of the case",
        ),
        (
            "intake = \"lines.jsonl\"\nraw = \"gone\"\ntaken = []\nheld = 0\n",
            "no folder of the case",
        ),
        (
            "import = \"lines.jsonl\"\nraw = \"raw\"\ntaken = []\nheld = 0\n",
            "`raw` is no import's",
        ),
        ("intake = \"lines.jsonl\"\nheld = 0\n", "`taken`"),
    ] {
        let toml = format!("{head}{step}");
        let refused = read(&[
            ("case.toml", &toml),
            ("first.sh", SCRIPT),
            ("lines.jsonl", "\n"),
            ("raw/notes.md", "notes\n"),
        ])
        .unwrap_err();
        assert!(refused.contains(says), "{step}: {refused}");
    }
}

#[test]
fn what_the_offers_name_of_each_capture() {
    let long = "a line of the notes\n".repeat(20);
    let lines = format!(
        "{}\n{}\n{}\n",
        serde_json::json!({"capture": {"key": "release/acme/1.0", "source": "release", "event_at": "2026-10-10T14:02:11Z", "raw": ["acme/notes.md"], "excerpt": {"text": long, "kind": "ask", "asserted_by": "acme", "assertion": "a", "subjects": [], "argues_with": [], "position": "p"}}}),
        serde_json::json!({"capture": {"key": "release/acme/0.9", "source": "release", "event_at": "2026-10-10T14:02:11Z", "excerpt": {"text": "short", "kind": "ask", "asserted_by": "acme", "assertion": "a", "subjects": [], "argues_with": [], "position": "p"}}}),
        serde_json::json!({"capture": {"key": "release/acme/1.0", "source": "release", "event_at": "2026-10-10T14:02:11Z", "raw": ["other.md"]}}),
    );
    let case = read(&[
        (
            "case.toml",
            "satisfies = [17]\nlimit = 200\n\n[[step]]\nintake = \"lines.jsonl\"\nraw = \"raw\"\ntaken = []\nheld = 0\n",
        ),
        ("lines.jsonl", &lines),
        ("raw/acme/notes.md", "the notes\n"),
    ])
    .unwrap();
    let offered = case.offered();
    assert_eq!(offered.len(), 2);
    let first = &offered["release/acme/1.0"];
    assert_eq!(
        first.raw,
        [("acme/notes.md".to_string(), b"the notes\n".to_vec())]
    );
    assert_eq!(first.whole_excerpt.as_deref(), Some(long.as_str()));
    assert_eq!(offered["release/acme/0.9"].whole_excerpt, None);
}
