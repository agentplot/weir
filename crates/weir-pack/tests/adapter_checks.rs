//! The adapter checks, held to "The adapter checks hold a run's lines to
//! the contract" (adapter-checks; 1 to 12; C.1).

use std::collections::BTreeMap;
use std::fs;

use weir_pack::{check_lines, Breach, RawPlace};

fn offer(key: &str, more: &str) -> String {
    format!(
        r#"{{"capture": {{"key": "{key}", "source": "release", "event_at": "2026-10-10T14:02:11Z"{more}}}}}"#
    )
}

const EXCERPT: &str = r#""kind": "ask", "asserted_by": "acme", "assertion": "acme 1 is out.", "subjects": [], "argues_with": [], "position": "the notes""#;

fn raw_files(paths: &[&str]) -> BTreeMap<String, Vec<u8>> {
    paths
        .iter()
        .map(|path| (path.to_string(), b"notes".to_vec()))
        .collect()
}

/// The breaches as (line, clause), in order.
fn named(breaches: &[Breach]) -> Vec<(Option<usize>, u32)> {
    breaches
        .iter()
        .map(|breach| (breach.line, breach.clause))
        .collect()
}

#[test]
fn a_run_that_keeps_the_contract() {
    let raw = raw_files(&["a/notes.md", "b/notes.md", "c/notes.md"]);
    let lines = [
        offer("release/acme/a", r#", "raw": ["a/notes.md"]"#),
        offer("release/acme/b", r#", "raw": ["b/notes.md"]"#),
        offer(
            "release/acme/c",
            &format!(
                r#", "raw": ["./c/notes.md"], "excerpt": {{"text": "acme 1 is out.", {EXCERPT}}}"#
            ),
        ),
        r#"{"cursor": "c2"}"#.to_string(),
        r#"{"done": {"listed": 3, "failed": []}}"#.to_string(),
    ];
    assert_eq!(check_lines(&lines, RawPlace::Files(&raw), true), []);
}

#[test]
fn two_offers_under_one_key() {
    let lines = [
        r#"{"cursor": "c1"}"#.to_string(),
        offer("release/acme/v1", ""),
        offer("release/acme/v2", ""),
        "not json".to_string(),
        offer("release/acme/v1", "").replace("2026-10-10T14:02:11Z", "2026-10-11T09:00:00Z"),
        r#"{"done": {"listed": 3, "failed": []}}"#.to_string(),
    ];
    let breaches = check_lines(&lines, RawPlace::Files(&BTreeMap::new()), true);
    assert_eq!(named(&breaches), [(Some(4), 1), (Some(5), 3)]);
    assert!(
        breaches[1]
            .why
            .contains("`release/acme/v1`, as line 2 does"),
        "{}",
        breaches[1]
    );
    // The same event offered twice, one instant written two ways, is one
    // event.
    let again = [
        offer("release/acme/v1", ""),
        offer("release/acme/v1", "").replace("2026-10-10T14:02:11Z", "2026-10-10T16:02:11+02:00"),
    ];
    assert_eq!(
        check_lines(&again, RawPlace::Files(&BTreeMap::new()), false),
        []
    );
}

#[test]
fn success_with_no_completion() {
    let lines = [
        offer("release/acme/v1", ""),
        r#"{"cursor": "c1"}"#.to_string(),
    ];
    let breaches = check_lines(&lines, RawPlace::Files(&BTreeMap::new()), true);
    assert_eq!(named(&breaches), [(None, 9)]);
    assert_eq!(
        breaches[0].to_string(),
        "the run breaks 9: it exited reporting success with no completion"
    );
    // A run that reports failure is incomplete, and breaks nothing for it.
    assert_eq!(
        check_lines(&lines, RawPlace::Files(&BTreeMap::new()), false),
        []
    );
}

#[test]
fn an_adapter_that_cut_its_own_excerpt() {
    let text =
        "acme 1 is out.\\n[cut: 12 lines and 4096 bytes left out, banked whole with the capture]";
    let lines = [
        offer(
            "release/acme/v1",
            &format!(r#", "excerpt": {{"text": "{text}", {EXCERPT}}}"#),
        ),
        offer(
            "release/acme/v2",
            &format!(r#", "excerpt": {{"text": "[cut: as the notes say]", {EXCERPT}}}"#),
        ),
    ];
    let breaches = check_lines(&lines, RawPlace::Files(&BTreeMap::new()), false);
    assert_eq!(named(&breaches), [(Some(1), 12)]);
}

#[test]
fn every_breach_of_a_line_names_its_clause() {
    let lines = [
        r#"{"cursor": "c1", "done": {"listed": 0, "failed": []}}"#.to_string(),
        r#"{"capture": {"source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#.to_string(),
        offer("captures/x", ""),
        offer("release/acme/v1", "").replace("14:02:11Z", "14:02:11"),
        offer("release/acme/v2", r#", "raw": ["../notes.md"]"#),
        offer("release/acme/v3", r#", "raw": ["missing.md"]"#),
        offer(
            "release/acme/v4",
            &format!(r#", "excerpt": {{"text": "", {EXCERPT}}}"#),
        ),
        offer(
            "release/acme/v5",
            r#", "excerpt": {"text": "acme", "kind": "ask"}"#,
        ),
        r#"{"done": {"listed": 8, "failed": [{"what": "github:acme"}]}}"#.to_string(),
        r#"{"done": {"listed": 8, "failed": [{"what": "", "why": "HTTP 502"}]}}"#.to_string(),
        offer("release/acme/v6", ""),
        r#"{"done": {"listed": 8, "failed": []}}"#.to_string(),
    ];
    let breaches = check_lines(&lines, RawPlace::Files(&BTreeMap::new()), true);
    assert_eq!(
        named(&breaches),
        [
            (Some(1), 1),
            (Some(2), 2),
            (Some(3), 4),
            (Some(4), 5),
            (Some(5), 6),
            (Some(6), 6),
            (Some(7), 7),
            (Some(8), 7),
            (Some(9), 9),
            (Some(10), 9),
            (Some(11), 9),
            (Some(12), 9),
        ],
        "{breaches:#?}"
    );
    assert!(breaches[2].why.contains("`captures`"), "{}", breaches[2]);
    assert!(
        breaches[8].why.contains("`done.failed[0].why` is missing"),
        "{}",
        breaches[8]
    );
    assert!(
        breaches[9].why.contains("`done.failed[0].what` is empty"),
        "{}",
        breaches[9]
    );
    assert!(
        breaches[10].why.contains("after line 10's completion"),
        "{}",
        breaches[10]
    );
    assert!(
        breaches[11].why.contains("a second completion"),
        "{}",
        breaches[11]
    );
}

#[test]
fn raw_material_read_from_a_folder() {
    let place = tempfile::tempdir().unwrap();
    fs::create_dir_all(place.path().join("m")).unwrap();
    fs::write(place.path().join("m/notes.md"), "notes").unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.md"), "elsewhere").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        outside.path().join("secret.md"),
        place.path().join("link.md"),
    )
    .unwrap();
    let lines = [
        offer("release/acme/v1", r#", "raw": ["m/notes.md"]"#),
        offer("release/acme/v2", r#", "raw": ["m"]"#),
        offer("release/acme/v3", r#", "raw": ["m/gone.md"]"#),
        offer("release/acme/v4", r#", "raw": ["link.md"]"#),
    ];
    let breaches = check_lines(&lines, RawPlace::Folder(place.path()), false);
    assert_eq!(
        named(&breaches),
        [(Some(2), 6), (Some(3), 6), (Some(4), 6)],
        "{breaches:#?}"
    );
    assert!(
        breaches[2].why.contains("outside the raw place"),
        "{}",
        breaches[2]
    );
}
